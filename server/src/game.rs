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
    pub input_move_vec: Vec2,
    pub input_target_angle: f32,
    pub respawn_timer: f32,
}

pub struct GameWorld {
    pub players: HashMap<PlayerId, ConnectedPlayer>,
    pub aliens: Vec<Alien>,
    pub lasers: Vec<Laser>,
    pub minerals: Vec<Mineral>,
    pub loot_boxes: Vec<LootBox>,
    pub portals: Vec<Portal>,
    pub current_map: MapId,
    pub space_base: SpaceBase,
    pub next_entity_id: u64,
    pub tick: u64,
    pub time_elapsed: f32,
    pub db_pool: DbPool,
}

pub type SharedWorld = Arc<RwLock<GameWorld>>;

impl GameWorld {
    pub fn new(db_pool: DbPool) -> Self {
        let space_base = SpaceBase {
            position: Vec2::new(0.0, 0.0),
            radius: BASE_RADIUS,
            name: "Station Spatiale Alpha".to_string(),
        };

        let mut world = Self {
            players: HashMap::new(),
            aliens: Vec::new(),
            lasers: Vec::new(),
            minerals: Vec::new(),
            loot_boxes: Vec::new(),
            portals: Vec::new(),
            current_map: MapId::Map1_1,
            space_base,
            next_entity_id: 1,
            tick: 0,
            time_elapsed: 0.0,
            db_pool,
        };

        world.init_world_entities();
        world
    }

    pub fn init_world_entities(&mut self) {
        let mut rng = rand::thread_rng();

        // 1. Minerals
        let mineral_dist = [
            (MineralType::Prometium, 40),
            (MineralType::Endurium, 25),
            (MineralType::Terbium, 15),
            (MineralType::Seprom, 8),
        ];

        for (m_type, count) in mineral_dist {
            for _ in 0..count {
                let dist = rng.gen_range(400.0..1800.0);
                let angle = rng.gen_range(0.0..std::f32::consts::TAU);
                self.minerals.push(Mineral {
                    id: self.next_entity_id,
                    mineral_type: m_type,
                    position: Vec2::new(angle.cos() * dist, angle.sin() * dist),
                    health: m_type.max_health(),
                    max_health: m_type.max_health(),
                });
                self.next_entity_id += 1;
            }
        }

        // 2. Aliens
        // Streuners (Map 1-1: 6 Streuners maximum, peaceful unless attacked)
        for _ in 0..6 {
            let dist = rng.gen_range(500.0..1200.0);
            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
            self.aliens.push(Alien {
                id: self.next_entity_id,
                alien_type: AlienType::Streuner,
                position: Vec2::new(angle.cos() * dist, angle.sin() * dist),
                velocity: Vec2::ZERO,
                rotation: angle,
                health: AlienType::Streuner.max_health(),
                max_health: AlienType::Streuner.max_health(),
                shield: AlienType::Streuner.max_shield(),
                max_shield: AlienType::Streuner.max_shield(),
                target_player_id: None,
                last_shot_time: 0.0,
            });
            self.next_entity_id += 1;
        }

        // 3. Portals
        // Jump Gate 1-2
        self.portals.push(Portal {
            id: self.next_entity_id,
            position: Vec2::new(1400.0, 1400.0),
            radius: 55.0,
            portal_type: PortalType::MapJump {
                target_map: MapId::Map1_2,
                target_pos: Vec2::new(-1300.0, -1300.0),
            },
        });
        self.next_entity_id += 1;

        // Jump Gate 4-4 (PvP)
        self.portals.push(Portal {
            id: self.next_entity_id,
            position: Vec2::new(-1400.0, -1400.0),
            radius: 55.0,
            portal_type: PortalType::MapJump {
                target_map: MapId::Map4_4,
                target_pos: Vec2::new(0.0, 0.0),
            },
        });
        self.next_entity_id += 1;

        // Dynamic Event Rifts
        self.portals.push(Portal {
            id: self.next_entity_id,
            position: Vec2::new(0.0, -1350.0),
            radius: 65.0,
            portal_type: PortalType::EventRift {
                event_type: EventRiftType::RedBoss,
                time_left: 360.0,
            },
        });
        self.next_entity_id += 1;

        self.portals.push(Portal {
            id: self.next_entity_id,
            position: Vec2::new(-1350.0, 0.0),
            radius: 65.0,
            portal_type: PortalType::EventRift {
                event_type: EventRiftType::GreenMining,
                time_left: 240.0,
            },
        });
        self.next_entity_id += 1;
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
                input_move_vec: Vec2::ZERO,
                input_target_angle: 0.0,
                respawn_timer: 0.0,
            },
        );

        let join_msg = ServerMessage::PlayerJoined { id, username: name };
        self.broadcast_message(&join_msg);
    }

    pub fn remove_player(&mut self, player_id: PlayerId) {
        if let Some(player) = self.players.remove(&player_id) {
            let pool = self.db_pool.clone();
            let save_data = player.ship.to_save_data();
            tokio::spawn(async move {
                let _ = crate::db::save_player_progression(&pool, player_id, &save_data);
            });

            let left_msg = ServerMessage::PlayerLeft { id: player_id };
            self.broadcast_message(&left_msg);
        }
    }

    pub fn handle_sync_progression(&mut self, player_id: PlayerId, save: PlayerSaveData) {
        if let Some(p) = self.players.get_mut(&player_id) {
            if save.level > p.ship.level 
                || save.xp > p.ship.xp 
                || save.credits > p.ship.credits 
                || (p.ship.level == 1 && p.ship.xp == 0 && (save.credits != 1000 || save.cargo.used_capacity() > 0 || save.talent_points > 0)) {
                p.ship.apply_save_data(&save);
                let pool = self.db_pool.clone();
                let save_clone = p.ship.to_save_data();
                tokio::spawn(async move {
                    let _ = crate::db::save_player_progression(&pool, player_id, &save_clone);
                });
            }
        }
    }

    pub fn handle_input(&mut self, player_id: PlayerId, thrust: bool, move_vec: Vec2, target_angle: f32) {
        if let Some(p) = self.players.get_mut(&player_id) {
            p.input_thrust = thrust;
            p.input_move_vec = move_vec;
            p.input_target_angle = target_angle;
        }
    }

    pub fn handle_shoot(&mut self, player_id: PlayerId) {
        if let Some(p) = self.players.get_mut(&player_id) {
            if !p.ship.is_alive || p.ship.is_in_safe_zone {
                return;
            }

            let talent_rate_bonus = 1.0 + (p.ship.talents.combat_fire_rate as f32 * 0.05);
            let cooldown = p.ship.ship_class.laser_cooldown() / talent_rate_bonus;

            if self.time_elapsed - p.last_shot_time >= cooldown {
                p.last_shot_time = self.time_elapsed;

                let dir = Vec2::new(p.ship.rotation.cos(), p.ship.rotation.sin());
                let spawn_pos = p.ship.position + dir * (SHIP_RADIUS + 4.0);

                let talent_dmg_bonus = 1.0 + (p.ship.talents.combat_laser_dmg as f32 * 0.06);
                let dmg = p.ship.ship_class.base_laser_damage() * talent_dmg_bonus;

                let laser = Laser {
                    id: self.next_entity_id,
                    shooter_id: player_id,
                    is_alien: false,
                    position: spawn_pos,
                    velocity: dir * LASER_SPEED + p.ship.velocity * 0.3,
                    lifetime: LASER_LIFETIME,
                    damage: dmg,
                    color_rgba: match p.ship.ship_class {
                        ShipClass::Combat => [0.0, 0.9, 1.0, 1.0],
                        ShipClass::Minier => [1.0, 0.75, 0.1, 1.0],
                        ShipClass::Transport => [0.2, 0.7, 1.0, 1.0],
                        ShipClass::Exploration => [0.2, 1.0, 0.5, 1.0],
                    },
                };
                self.next_entity_id += 1;
                self.lasers.push(laser);
            }
        }
    }

    pub fn handle_mining(&mut self, player_id: PlayerId, mineral_id: u64, is_start: bool) {
        if let Some(p) = self.players.get_mut(&player_id) {
            if is_start && p.ship.ship_class.can_mine() {
                p.ship.is_mining = true;
                p.ship.mining_target = Some(mineral_id);
            } else {
                p.ship.is_mining = false;
                p.ship.mining_target = None;
            }
        }
    }

    pub fn handle_select_class(&mut self, player_id: PlayerId, class: ShipClass) {
        if let Some(p) = self.players.get_mut(&player_id) {
            p.ship.ship_class = class;
            p.ship.max_health = class.base_health();
            p.ship.health = class.base_health();
            p.ship.max_shield = class.base_shield();
            p.ship.shield = class.base_shield();
            p.ship.apply_talent_bonuses();
        }
    }

    pub fn handle_upgrade_talent(&mut self, player_id: PlayerId, talent_idx: u32) {
        if let Some(p) = self.players.get_mut(&player_id) {
            if p.ship.talent_points > 0 {
                p.ship.talent_points -= 1;
                match talent_idx {
                    0 => p.ship.talents.combat_laser_dmg += 1,
                    1 => p.ship.talents.combat_fire_rate += 1,
                    2 => p.ship.talents.defense_shield_max += 1,
                    3 => p.ship.talents.defense_regen += 1,
                    4 => p.ship.talents.logistics_cargo += 1,
                    5 => p.ship.talents.logistics_mining_speed += 1,
                    _ => {}
                }
                p.ship.apply_talent_bonuses();
            }
        }
    }

    pub fn handle_sell_cargo(&mut self, player_id: PlayerId) {
        if let Some(p) = self.players.get_mut(&player_id) {
            if p.ship.is_in_safe_zone {
                let credits = p.ship.cargo.sell_all_minerals();
                p.ship.credits += credits;
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
                    rng.gen_range(-200.0..200.0),
                    rng.gen_range(-200.0..200.0),
                );
            }
        }
    }

    pub fn tick_simulation(&mut self) {
        self.tick += 1;
        self.time_elapsed += TICK_DT;

        let base_pos = self.space_base.position;
        let base_rad = self.space_base.radius;

        // 1. Update Players Physics, Safe Zone & Shield Regen
        for player in self.players.values_mut() {
            if player.ship.is_alive {
                apply_ship_physics(
                    &mut player.ship,
                    player.input_move_vec,
                    player.input_target_angle,
                    TICK_DT,
                );

                let in_safe = player.ship.position.distance_to(base_pos) <= base_rad;
                player.ship.is_in_safe_zone = in_safe;

                if in_safe {
                    let regen = 25.0 * TICK_DT;
                    player.ship.health = (player.ship.health + regen).min(player.ship.max_health);
                    player.ship.shield = (player.ship.shield + regen * 1.5).min(player.ship.max_shield);
                } else {
                    let regen = 4.0 * TICK_DT * (1.0 + player.ship.talents.defense_regen as f32 * 0.15);
                    player.ship.shield = (player.ship.shield + regen).min(player.ship.max_shield);
                }
            } else if player.respawn_timer > 0.0 {
                player.respawn_timer -= TICK_DT;
            }
        }

        // Continuous Alien Population Replenishment
        let mut rng = rand::thread_rng();
        if self.current_map == MapId::Map1_1 {
            self.aliens.retain(|a| a.alien_type == AlienType::Streuner);
            let streuner_count = self.aliens.iter().filter(|a| a.health > 0.0 && a.alien_type == AlienType::Streuner).count();
            if streuner_count < 6 {
                let dist = rng.gen_range(500.0..1200.0);
                let angle = rng.gen_range(0.0..std::f32::consts::TAU);
                self.aliens.push(Alien {
                    id: self.next_entity_id,
                    alien_type: AlienType::Streuner,
                    position: Vec2::new(angle.cos() * dist, angle.sin() * dist),
                    velocity: Vec2::ZERO,
                    rotation: angle,
                    health: AlienType::Streuner.max_health(),
                    max_health: AlienType::Streuner.max_health(),
                    shield: AlienType::Streuner.max_shield(),
                    max_shield: AlienType::Streuner.max_shield(),
                    target_player_id: None,
                    last_shot_time: 0.0,
                });
                self.next_entity_id += 1;
            }
        } else {
            let streuner_count = self.aliens.iter().filter(|a| a.health > 0.0 && a.alien_type == AlienType::Streuner).count();
            let lordakia_count = self.aliens.iter().filter(|a| a.health > 0.0 && a.alien_type == AlienType::Lordakia).count();

            if streuner_count < 12 {
                let dist = rng.gen_range(500.0..1200.0);
                let angle = rng.gen_range(0.0..std::f32::consts::TAU);
                self.aliens.push(Alien {
                    id: self.next_entity_id,
                    alien_type: AlienType::Streuner,
                    position: Vec2::new(angle.cos() * dist, angle.sin() * dist),
                    velocity: Vec2::ZERO,
                    rotation: angle,
                    health: AlienType::Streuner.max_health(),
                    max_health: AlienType::Streuner.max_health(),
                    shield: AlienType::Streuner.max_shield(),
                    max_shield: AlienType::Streuner.max_shield(),
                    target_player_id: None,
                    last_shot_time: 0.0,
                });
                self.next_entity_id += 1;
            }

            if lordakia_count < 8 {
                let dist = rng.gen_range(1100.0..1800.0);
                let angle = rng.gen_range(0.0..std::f32::consts::TAU);
                self.aliens.push(Alien {
                    id: self.next_entity_id,
                    alien_type: AlienType::Lordakia,
                    position: Vec2::new(angle.cos() * dist, angle.sin() * dist),
                    velocity: Vec2::ZERO,
                    rotation: angle,
                    health: AlienType::Lordakia.max_health(),
                    max_health: AlienType::Lordakia.max_health(),
                    shield: AlienType::Lordakia.max_shield(),
                    max_shield: AlienType::Lordakia.max_shield(),
                    target_player_id: None,
                    last_shot_time: 0.0,
                });
                self.next_entity_id += 1;
            }
        }

        // 2. Alien AI Simulation
        let mut alien_lasers = Vec::new();
        let now = self.time_elapsed;

        for alien in &mut self.aliens {
            if alien.health <= 0.0 {
                continue;
            }

            // Keep aliens inside bounds and outside safe station
            let dist_from_center = alien.position.length();
            if dist_from_center > 1450.0 {
                let to_center = -alien.position.normalize();
                alien.rotation = to_center.y.atan2(to_center.x);
            } else if dist_from_center < 320.0 {
                let away = alien.position.normalize();
                alien.rotation = away.y.atan2(away.x);
            }

            // Map 1-1 Streuners are strictly non-aggressive unless attacked!
            let is_passive_streuner = self.current_map == MapId::Map1_1 && alien.alien_type == AlienType::Streuner;
            let mut target_ship_info = None;

            if let Some(target_id) = alien.target_player_id {
                if let Some(p) = self.players.get(&target_id) {
                    let d = alien.position.distance_to(p.ship.position);
                    // Disengage if safe zone, dead, or > 700m away
                    if p.ship.is_alive && !p.ship.is_in_safe_zone && d <= 700.0 {
                        target_ship_info = Some((target_id, p.ship.position, d));
                    } else {
                        alien.target_player_id = None;
                    }
                } else {
                    alien.target_player_id = None;
                }
            } else if !is_passive_streuner {
                // Aggressive aliens scan for closest player
                let mut closest_player = None;
                let mut min_dist = alien.alien_type.aggro_range();

                for (p_id, p) in &self.players {
                    if p.ship.is_alive && !p.ship.is_in_safe_zone {
                        let d = alien.position.distance_to(p.ship.position);
                        if d < min_dist {
                            min_dist = d;
                            closest_player = Some((*p_id, p.ship.position, d));
                        }
                    }
                }
                target_ship_info = closest_player;
            }

            if let Some((_target_id, target_pos, target_dist)) = target_ship_info {
                let dir = (target_pos - alien.position).normalize();
                alien.rotation = dir.y.atan2(dir.x);
                alien.velocity = dir * alien.alien_type.speed();
                alien.position += alien.velocity * TICK_DT;

                if now - alien.last_shot_time >= alien.alien_type.laser_cooldown() && target_dist < 420.0 {
                    alien.last_shot_time = now;
                    let spawn_pos = alien.position + dir * 20.0;
                    alien_lasers.push(Laser {
                        id: self.next_entity_id,
                        shooter_id: alien.id,
                        is_alien: true,
                        position: spawn_pos,
                        velocity: dir * (LASER_SPEED * 0.75),
                        lifetime: LASER_LIFETIME,
                        damage: alien.alien_type.laser_damage(),
                        color_rgba: [1.0, 0.2, 0.25, 1.0],
                    });
                    self.next_entity_id += 1;
                }
            } else {
                alien.rotation += (alien.id as f32 * 0.1 + now * 0.2).sin() * TICK_DT;
                let forward = Vec2::new(alien.rotation.cos(), alien.rotation.sin());
                alien.position += forward * (alien.alien_type.speed() * 0.3 * TICK_DT);
            }
        }

        self.lasers.extend(alien_lasers);

        // 3. Update Lasers
        for laser in &mut self.lasers {
            laser.position += laser.velocity * TICK_DT;
            laser.lifetime -= TICK_DT;
        }

        // 4. Laser Collisions (Aliens & PvP)
        let mut hit_alien_ids = Vec::new();
        let mut kills = Vec::new();

        for laser in &mut self.lasers {
            if laser.lifetime <= 0.0 {
                continue;
            }

            if !laser.is_alien {
                // Player laser hits aliens
                for alien in &mut self.aliens {
                    if alien.health > 0.0 && laser.position.distance_to(alien.position) < 26.0 {
                        laser.lifetime = 0.0;
                        alien.target_player_id = Some(laser.shooter_id);
                        let dmg = laser.damage;
                        if alien.shield > 0.0 {
                            let absorbed = dmg.min(alien.shield);
                            alien.shield -= absorbed;
                            let remaining = dmg - absorbed;
                            alien.health -= remaining;
                        } else {
                            alien.health -= dmg;
                        }

                        if alien.health <= 0.0 {
                            hit_alien_ids.push((alien.id, laser.shooter_id));
                        }
                        break;
                    }
                }

                // PvP in Map 4-4
                if self.current_map.is_pvp() {
                    for (target_id, target_player) in self.players.iter_mut() {
                        if *target_id != laser.shooter_id && target_player.ship.is_alive && !target_player.ship.is_in_safe_zone {
                            if laser.position.distance_to(target_player.ship.position) <= SHIP_RADIUS + 3.0 {
                                laser.lifetime = 0.0;
                                let mut rem_damage = laser.damage;
                                if target_player.ship.shield > 0.0 {
                                    let abs = rem_damage.min(target_player.ship.shield);
                                    target_player.ship.shield -= abs;
                                    rem_damage -= abs;
                                }
                                target_player.ship.health -= rem_damage;
                                if target_player.ship.health <= 0.0 {
                                    target_player.ship.health = 0.0;
                                    target_player.ship.is_alive = false;
                                    target_player.respawn_timer = 3.0;
                                    kills.push((*target_id, laser.shooter_id));
                                }
                                break;
                            }
                        }
                    }
                }
            } else {
                // Alien laser hits players
                for (target_id, target_player) in self.players.iter_mut() {
                    if target_player.ship.is_alive && !target_player.ship.is_in_safe_zone {
                        if laser.position.distance_to(target_player.ship.position) <= SHIP_RADIUS + 3.0 {
                            laser.lifetime = 0.0;
                            let mut rem_damage = laser.damage;
                            if target_player.ship.shield > 0.0 {
                                let abs = rem_damage.min(target_player.ship.shield);
                                target_player.ship.shield -= abs;
                                rem_damage -= abs;
                            }
                            target_player.ship.health -= rem_damage;
                            if target_player.ship.health <= 0.0 {
                                target_player.ship.health = 0.0;
                                target_player.ship.is_alive = false;
                                target_player.respawn_timer = 3.0;
                                kills.push((*target_id, 0));
                            }
                            break;
                        }
                    }
                }
            }
        }

        self.lasers.retain(|l| l.lifetime > 0.0);

        // Process Dead Aliens & Drop Loot
        for (dead_id, killer_id) in hit_alien_ids {
            if let Some(pos) = self.aliens.iter().position(|a| a.id == dead_id) {
                let alien = self.aliens.remove(pos);
                let xp = alien.alien_type.xp_reward();
                let creds = alien.alien_type.credits_reward();

                if let Some(killer) = self.players.get_mut(&killer_id) {
                    killer.ship.add_xp(xp);
                    killer.ship.credits += creds;
                }

                self.loot_boxes.push(LootBox {
                    id: self.next_entity_id,
                    position: alien.position,
                    credits: creds / 2,
                    mineral: Some((MineralType::Prometium, 3)),
                    scrap: 2,
                    plasma_cores: if alien.alien_type == AlienType::Sibelon { 2 } else { 0 },
                    lifetime: 60.0,
                    owner_id: Some(killer_id),
                });
                self.next_entity_id += 1;
            }
        }

        // Process PvP Kills & Drop Player Cargo
        for (victim_id, killer_id) in kills {
            let kill_msg = ServerMessage::PlayerKilled { victim_id, killer_id };
            self.broadcast_message(&kill_msg);

            let (drop_pos, drop_creds, drop_scrap) = if let Some(victim) = self.players.get_mut(&victim_id) {
                let creds = (victim.ship.credits / 10).min(500);
                victim.ship.credits = victim.ship.credits.saturating_sub(creds);
                let scrap = victim.ship.cargo.scrap / 2;
                victim.ship.cargo.scrap -= scrap;
                (victim.ship.position, creds, scrap)
            } else {
                (Vec2::ZERO, 0, 0)
            };

            self.loot_boxes.push(LootBox {
                id: self.next_entity_id,
                position: drop_pos,
                credits: drop_creds.max(50),
                mineral: Some((MineralType::Prometium, 2)),
                scrap: drop_scrap.max(1),
                plasma_cores: 0,
                lifetime: 60.0,
                owner_id: if killer_id != 0 { Some(killer_id) } else { None },
            });
            self.next_entity_id += 1;

            if let Some(killer) = self.players.get_mut(&killer_id) {
                killer.ship.add_xp(200);
                killer.ship.credits += 800;
            }
        }

        // 5. Loot Boxes Collection
        let mut collected_box_ids = Vec::new();
        for loot in &mut self.loot_boxes {
            loot.lifetime -= TICK_DT;
            for player in self.players.values_mut() {
                let can_collect = match loot.owner_id {
                    Some(owner) => owner == player.ship.id,
                    None => true,
                };
                if can_collect && player.ship.is_alive && loot.position.distance_to(player.ship.position) < LOOTBOX_RADIUS + SHIP_RADIUS {
                    collected_box_ids.push(loot.id);
                    player.ship.credits += loot.credits;
                    player.ship.cargo.add_scrap(loot.scrap);
                    if loot.plasma_cores > 0 {
                        player.ship.cargo.add_plasma_core(loot.plasma_cores);
                    }
                    if let Some((m, amt)) = loot.mineral {
                        let _ = player.ship.cargo.add_mineral(m, amt);
                    }
                    break;
                }
            }
        }
        self.loot_boxes.retain(|lb| lb.lifetime > 0.0 && !collected_box_ids.contains(&lb.id));

        // 6. Mining Simulation
        for player in self.players.values_mut() {
            if player.ship.is_mining && player.ship.is_alive {
                if let Some(target_id) = player.ship.mining_target {
                    if let Some(mineral) = self.minerals.iter_mut().find(|m| m.id == target_id) {
                        let dist = player.ship.position.distance_to(mineral.position);
                        if dist <= MINING_RANGE {
                            let talent_bonus = 1.0 + (player.ship.talents.logistics_mining_speed as f32 * 0.25);
                            mineral.health -= MINING_DAMAGE_PER_SEC * talent_bonus * TICK_DT;

                            if mineral.health <= 0.0 {
                                let m_type = mineral.mineral_type;
                                player.ship.cargo.add_mineral(m_type, 1);
                                player.ship.add_xp(m_type.value() * 2);

                                mineral.health = mineral.max_health;
                                let mut rng = rand::thread_rng();
                                let d = rng.gen_range(500.0..1600.0);
                                let a = rng.gen_range(0.0..std::f32::consts::TAU);
                                mineral.position = Vec2::new(a.cos() * d, a.sin() * d);

                                player.ship.is_mining = false;
                                player.ship.mining_target = None;
                            }
                        } else {
                            player.ship.is_mining = false;
                            player.ship.mining_target = None;
                        }
                    } else {
                        player.ship.is_mining = false;
                        player.ship.mining_target = None;
                    }
                }
            }
        }

        // Periodic auto-save every 5 seconds (5 * TICK_RATE ticks)
        if self.tick % (TICK_RATE * 5) == 0 {
            for (player_id, p) in &self.players {
                let pool = self.db_pool.clone();
                let pid = *player_id;
                let save_data = p.ship.to_save_data();
                tokio::spawn(async move {
                    let _ = crate::db::save_player_progression(&pool, pid, &save_data);
                });
            }
        }

        // Broadcast World Snapshot
        self.broadcast_snapshot();
    }

    pub fn create_snapshot(&self) -> WorldSnapshot {
        WorldSnapshot {
            tick: self.tick,
            server_time_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            current_map: self.current_map,
            players: self.players.values().map(|p| p.ship.clone()).collect(),
            aliens: self.aliens.clone(),
            lasers: self.lasers.clone(),
            minerals: self.minerals.clone(),
            loot_boxes: self.loot_boxes.clone(),
            portals: self.portals.clone(),
        }
    }

    pub fn broadcast_snapshot(&self) {
        let snapshot = self.create_snapshot();
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
