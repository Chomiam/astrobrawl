use astrobrawl_shared::*;
use crate::db::DbPool;
use rand::Rng;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

pub struct ConnectedPlayer {
    pub ship: PlayerShip,
    pub tx: mpsc::UnboundedSender<Vec<u8>>,
    pub last_shot_time: f32,
    pub input_thrust: bool,
    pub input_target_angle: f32,
    pub respawn_timer: f32,
}

pub struct GameWorld {
    pub players: HashMap<PlayerId, ConnectedPlayer>,
    pub lasers: Vec<Laser>,
    pub minerals: Vec<Mineral>,
    pub next_entity_id: u64,
    pub tick: u64,
    pub time_elapsed: f32,
    pub db_pool: DbPool,
}

pub type SharedWorld = Arc<RwLock<GameWorld>>;

impl GameWorld {
    pub fn new(db_pool: DbPool) -> Self {
        let mut world = Self {
            players: HashMap::new(),
            lasers: Vec::new(),
            minerals: Vec::new(),
            next_entity_id: 1,
            tick: 0,
            time_elapsed: 0.0,
            db_pool,
        };

        // Populate initial minerals
        world.populate_minerals();
        world
    }

    pub fn populate_minerals(&mut self) {
        let mut rng = rand::thread_rng();
        let half_w = WORLD_WIDTH * 0.45;
        let half_h = WORLD_HEIGHT * 0.45;

        while self.minerals.len() < MAX_MINERALS_ON_MAP {
            let m_type = match rng.gen_range(0..100) {
                0..=59 => MineralType::Prometium,
                60..=89 => MineralType::Endurium,
                _ => MineralType::Terbium,
            };

            let pos = Vec2::new(
                rng.gen_range(-half_w..half_w),
                rng.gen_range(-half_h..half_h),
            );

            self.minerals.push(Mineral {
                id: self.next_entity_id,
                mineral_type: m_type,
                position: pos,
                value: m_type.value(),
            });
            self.next_entity_id += 1;
        }
    }

    pub fn add_player(&mut self, player_ship: PlayerShip, tx: mpsc::UnboundedSender<Vec<u8>>) {
        let id = player_ship.id;
        let name = player_ship.username.clone();

        self.players.insert(
            id,
            ConnectedPlayer {
                ship: player_ship,
                tx,
                last_shot_time: 0.0,
                input_thrust: false,
                input_target_angle: 0.0,
                respawn_timer: 0.0,
            },
        );

        // Notify other players
        let join_msg = ServerMessage::PlayerJoined { id, username: name };
        self.broadcast_message(&join_msg);
    }

    pub fn remove_player(&mut self, player_id: PlayerId) {
        if let Some(player) = self.players.remove(&player_id) {
            // Save player stats in DB asynchronously
            let pool = self.db_pool.clone();
            let c = player.ship.credits;
            let m = player.ship.minerals;
            let s = player.ship.score;
            tokio::spawn(async move {
                let _ = crate::db::save_player_stats(&pool, player_id, c, m, s);
            });

            let left_msg = ServerMessage::PlayerLeft { id: player_id };
            self.broadcast_message(&left_msg);
        }
    }

    pub fn handle_input(&mut self, player_id: PlayerId, thrust: bool, target_angle: f32) {
        if let Some(p) = self.players.get_mut(&player_id) {
            p.input_thrust = thrust;
            p.input_target_angle = target_angle;
        }
    }

    pub fn handle_shoot(&mut self, player_id: PlayerId) {
        if let Some(p) = self.players.get_mut(&player_id) {
            if !p.ship.is_alive {
                return;
            }

            if self.time_elapsed - p.last_shot_time >= LASER_COOLDOWN {
                p.last_shot_time = self.time_elapsed;

                // Spawn laser from front of ship
                let dir = Vec2::new(p.ship.rotation.cos(), p.ship.rotation.sin());
                let spawn_pos = p.ship.position + dir * (SHIP_RADIUS + 4.0);
                let laser_vel = dir * LASER_SPEED + p.ship.velocity * 0.3;

                let laser = Laser {
                    id: self.next_entity_id,
                    shooter_id: player_id,
                    position: spawn_pos,
                    velocity: laser_vel,
                    lifetime: LASER_LIFETIME,
                };
                self.next_entity_id += 1;
                self.lasers.push(laser);
            }
        }
    }

    pub fn handle_respawn(&mut self, player_id: PlayerId) {
        let mut rng = rand::thread_rng();
        if let Some(p) = self.players.get_mut(&player_id) {
            if !p.ship.is_alive && p.respawn_timer <= 0.0 {
                p.ship.is_alive = true;
                p.ship.health = p.ship.max_health;
                p.ship.shield = p.ship.max_shield;
                p.ship.velocity = Vec2::ZERO;
                p.ship.position = Vec2::new(
                    rng.gen_range(-400.0..400.0),
                    rng.gen_range(-400.0..400.0),
                );
            }
        }
    }

    pub fn tick_simulation(&mut self) {
        self.tick += 1;
        self.time_elapsed += TICK_DT;

        // 1. Update Players Physics & Shields
        for player in self.players.values_mut() {
            if player.ship.is_alive {
                apply_ship_physics(
                    &mut player.ship,
                    player.input_thrust,
                    player.input_target_angle,
                    TICK_DT,
                );

                // Shield passive regeneration
                if player.ship.shield < player.ship.max_shield {
                    player.ship.shield =
                        (player.ship.shield + SHIELD_REGEN_PER_SEC * TICK_DT).min(player.ship.max_shield);
                }
            } else {
                if player.respawn_timer > 0.0 {
                    player.respawn_timer -= TICK_DT;
                }
            }
        }

        // 2. Update Lasers
        for laser in &mut self.lasers {
            laser.position += laser.velocity * TICK_DT;
            laser.lifetime -= TICK_DT;
        }

        // 3. Laser collisions with players
        let mut dead_lasers = Vec::new();
        let mut kills = Vec::new();

        for (l_idx, laser) in self.lasers.iter().enumerate() {
            if laser.lifetime <= 0.0 {
                dead_lasers.push(l_idx);
                continue;
            }

            for (target_id, target_player) in self.players.iter_mut() {
                if *target_id != laser.shooter_id && target_player.ship.is_alive {
                    let dist = laser.position.distance_to(target_player.ship.position);
                    if dist <= SHIP_RADIUS + 3.0 {
                        dead_lasers.push(l_idx);

                        // Apply damage: shield takes damage first
                        let mut rem_damage = LASER_DAMAGE;
                        if target_player.ship.shield > 0.0 {
                            if target_player.ship.shield >= rem_damage {
                                target_player.ship.shield -= rem_damage;
                                rem_damage = 0.0;
                            } else {
                                rem_damage -= target_player.ship.shield;
                                target_player.ship.shield = 0.0;
                            }
                        }

                        target_player.ship.health -= rem_damage;

                        // Check ship destruction
                        if target_player.ship.health <= 0.0 {
                            target_player.ship.health = 0.0;
                            target_player.ship.is_alive = false;
                            target_player.respawn_timer = 3.0; // 3 sec before respawn allowed
                            kills.push((*target_id, laser.shooter_id));
                        }
                        break;
                    }
                }
            }
        }

        // Remove dead lasers (in reverse order to preserve indices)
        dead_lasers.sort_unstable();
        dead_lasers.dedup();
        for &idx in dead_lasers.iter().rev() {
            if idx < self.lasers.len() {
                self.lasers.swap_remove(idx);
            }
        }

        // Handle kills: award score & credits
        for (victim_id, killer_id) in kills {
            let kill_msg = ServerMessage::PlayerKilled {
                victim_id,
                killer_id,
            };
            self.broadcast_message(&kill_msg);

            if let Some(killer) = self.players.get_mut(&killer_id) {
                killer.ship.score += 250;
                killer.ship.credits += 500;

                let stats_msg = ServerMessage::StatsUpdated {
                    health: killer.ship.health,
                    shield: killer.ship.shield,
                    minerals: killer.ship.minerals,
                    credits: killer.ship.credits,
                    score: killer.ship.score,
                };
                let _ = killer.tx.send(serialize_packet(&stats_msg).unwrap_or_default());
            }
        }

        // 4. Mineral collection
        let mut collected_minerals = Vec::new();
        for (m_idx, mineral) in self.minerals.iter().enumerate() {
            for (player_id, player) in self.players.iter_mut() {
                if player.ship.is_alive {
                    let dist = mineral.position.distance_to(player.ship.position);
                    if dist <= MINERAL_COLLECT_RADIUS {
                        collected_minerals.push((
                            m_idx,
                            *player_id,
                            mineral.id,
                            mineral.mineral_type,
                            mineral.value,
                        ));
                        break;
                    }
                }
            }
        }

        // Process collected minerals
        if !collected_minerals.is_empty() {
            // Sort indices descending to swap_remove safely
            collected_minerals.sort_by(|a, b| b.0.cmp(&a.0));
            for (idx, player_id, mineral_id, m_type, val) in collected_minerals {
                if idx < self.minerals.len() {
                    self.minerals.swap_remove(idx);
                }

                if let Some(player) = self.players.get_mut(&player_id) {
                    player.ship.minerals += 1;
                    player.ship.credits += val;
                    player.ship.score += val * 2;

                    let col_msg = ServerMessage::MineralCollected {
                        player_id,
                        mineral_id,
                        mineral_type: m_type,
                        value: val,
                        total_minerals: player.ship.minerals,
                        total_credits: player.ship.credits,
                    };
                    self.broadcast_message(&col_msg);
                }
            }
        }

        // Respawn minerals if needed
        self.populate_minerals();

        // 5. Broadcast World Snapshot
        self.broadcast_snapshot();
    }

    pub fn broadcast_snapshot(&self) {
        let snapshot = WorldSnapshot {
            tick: self.tick,
            server_time_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            players: self.players.values().map(|p| p.ship.clone()).collect(),
            lasers: self.lasers.clone(),
            minerals: self.minerals.clone(),
        };

        let msg = ServerMessage::WorldSnapshot(snapshot);
        if let Ok(bytes) = serialize_packet(&msg) {
            for player in self.players.values() {
                let _ = player.tx.send(bytes.clone());
            }
        }
    }

    pub fn broadcast_message(&self, msg: &ServerMessage) {
        if let Ok(bytes) = serialize_packet(msg) {
            for player in self.players.values() {
                let _ = player.tx.send(bytes.clone());
            }
        }
    }
}
