use astrobrawl_shared::Vec2;
use astrobrawl_shared::*;
use macroquad::prelude::*;
use std::collections::HashMap;

fn to_mq(v: Vec2) -> macroquad::math::Vec2 {
    vec2(v.x, v.y)
}

// --- Native & WASM Network Bridge ---
#[cfg(target_arch = "wasm32")]
mod net {
    extern "C" {
        pub fn mq_ws_connect(url_ptr: *const u8, url_len: usize);
        pub fn mq_ws_is_connected() -> i32;
        pub fn mq_ws_send(ptr: *const u8, len: usize);
        pub fn mq_ws_recv(dest_ptr: *mut u8, max_len: usize) -> i32;
    }

    pub fn connect(url: &str) {
        unsafe { mq_ws_connect(url.as_ptr(), url.len()) };
    }

    pub fn is_connected() -> bool {
        unsafe { mq_ws_is_connected() == 1 }
    }

    pub fn send(bytes: &[u8]) {
        unsafe { mq_ws_send(bytes.as_ptr(), bytes.len()) };
    }

    pub fn try_recv() -> Option<Vec<u8>> {
        let mut buf = vec![0u8; 65536];
        let len = unsafe { mq_ws_recv(buf.as_mut_ptr(), buf.len()) };
        if len > 0 {
            buf.truncate(len as usize);
            Some(buf)
        } else {
            None
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod net {
    pub fn connect(_url: &str) {}
    pub fn is_connected() -> bool {
        false
    }
    pub fn send(_bytes: &[u8]) {}
    pub fn try_recv() -> Option<Vec<u8>> {
        None
    }
}

// --- Visual FX ---
struct Particle {
    pos: Vec2,
    vel: Vec2,
    color: Color,
    lifetime: f32,
    max_lifetime: f32,
    size: f32,
}

struct FloatText {
    text: String,
    pos: Vec2,
    color: Color,
    lifetime: f32,
}

struct Star {
    pos: Vec2,
    size: f32,
    color: Color,
    layer: f32,
}

// --- UI Overlay Modes ---
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActiveModal {
    None,
    Hangar,
    Talents,
    Crafting,
}

// --- Client Game State ---
struct GameClient {
    local_player_id: PlayerId,
    local_ship: PlayerShip,
    current_map: MapId,
    connected: bool,
    connecting: bool,
    status_text: String,

    // Multiplayer World Entities
    players: HashMap<PlayerId, PlayerShip>,
    aliens: Vec<Alien>,
    lasers: Vec<Laser>,
    minerals: Vec<Mineral>,
    loot_boxes: Vec<LootBox>,
    portals: Vec<Portal>,

    // Local Simulation
    is_offline_sim: bool,
    next_entity_id: u64,
    space_base: SpaceBase,
    mining_sound_timer: f32,

    // Camera & UI
    camera_pos: Vec2,
    last_ping_send: f64,
    ping_ms: u64,
    active_modal: ActiveModal,
    notification_text: String,
    notification_timer: f32,

    // Particles & Parallax
    particles: Vec<Particle>,
    float_texts: Vec<FloatText>,
    stars: Vec<Star>,
}

impl GameClient {
    fn new() -> Self {
        let mut stars = Vec::with_capacity(350);
        for i in 0..350 {
            let seed = (i as f32) * 12.9898;
            let x = ((seed.sin() * 43758.5453).fract() - 0.5) * WORLD_WIDTH * 1.5;
            let y = (((seed + 1.0).sin() * 43758.5453).fract() - 0.5) * WORLD_HEIGHT * 1.5;
            let layer = ((seed + 2.0).sin() * 43758.5453).fract() * 0.8 + 0.2;
            let size = 1.0 + layer * 2.0;
            let col = Color::new(0.65 + layer * 0.35, 0.75 + layer * 0.25, 1.0, 1.0);

            stars.push(Star {
                pos: Vec2::new(x, y),
                size,
                color: col,
                layer,
            });
        }

        let local_id = 1;
        let default_ship = PlayerShip::new(
            local_id,
            "Pilote Spatial".to_string(),
            ShipClass::Combat,
            Vec2::new(0.0, 0.0),
        );

        let space_base = SpaceBase {
            position: Vec2::new(0.0, 0.0),
            radius: BASE_RADIUS,
            name: "Station Stellaire Alpha".to_string(),
        };

        let mut client = Self {
            local_player_id: local_id,
            local_ship: default_ship,
            current_map: MapId::Map1_1,
            connected: false,
            connecting: false,
            status_text: "Mode Solo / Simulation Active".to_string(),

            players: HashMap::new(),
            aliens: Vec::new(),
            lasers: Vec::new(),
            minerals: Vec::new(),
            loot_boxes: Vec::new(),
            portals: Vec::new(),

            is_offline_sim: true,
            next_entity_id: 100,
            space_base,
            mining_sound_timer: 0.0,

            camera_pos: Vec2::ZERO,
            last_ping_send: 0.0,
            ping_ms: 0,
            active_modal: ActiveModal::None,
            notification_text: "Bienvenue dans AstroBrawl ! Rejoignez la base spatiale au centre.".to_string(),
            notification_timer: 6.0,

            particles: Vec::new(),
            float_texts: Vec::new(),
            stars,
        };

        // Initialize offline world entities
        client.init_offline_world();
        client
    }

    fn init_offline_world(&mut self) {
        self.minerals.clear();
        self.aliens.clear();
        self.portals.clear();
        self.loot_boxes.clear();

        // 1. Spawning Minerals with Rarity
        let mineral_distribution = [
            (MineralType::Prometium, 38), // Orange
            (MineralType::Endurium, 24),  // Cyan
            (MineralType::Terbium, 14),   // Green
            (MineralType::Seprom, 6),     // Purple / Prismatic
        ];

        for (m_type, count) in mineral_distribution {
            for i in 0..count {
                let seed = (self.next_entity_id as f32 + i as f32) * 45.67;
                let angle = (seed.sin() * 43758.5453).fract() * std::f32::consts::TAU;
                let dist = 420.0 + ((seed + 1.0).sin() * 43758.5453).fract().abs() * 1450.0;
                let pos = Vec2::new(angle.cos() * dist, angle.sin() * dist);

                self.minerals.push(Mineral {
                    id: self.next_entity_id,
                    mineral_type: m_type,
                    position: pos,
                    health: m_type.max_health(),
                    max_health: m_type.max_health(),
                });
                self.next_entity_id += 1;
            }
        }

        // 2. Spawning Aliens
        for i in 0..9 {
            let seed = (i as f32 + 10.0) * 31.41;
            let angle = (seed.sin() * 43758.5453).fract() * std::f32::consts::TAU;
            let dist = 550.0 + ((seed + 2.0).sin() * 43758.5453).fract().abs() * 650.0;
            let pos = Vec2::new(angle.cos() * dist, angle.sin() * dist);

            self.aliens.push(Alien {
                id: self.next_entity_id,
                alien_type: AlienType::Streuner,
                position: pos,
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

        // Lordakias (Fast fighters in outer ring)
        for i in 0..5 {
            let seed = (i as f32 + 30.0) * 17.89;
            let angle = (seed.sin() * 43758.5453).fract() * std::f32::consts::TAU;
            let dist = 1250.0 + ((seed + 1.0).sin() * 43758.5453).fract().abs() * 600.0;
            let pos = Vec2::new(angle.cos() * dist, angle.sin() * dist);

            self.aliens.push(Alien {
                id: self.next_entity_id,
                alien_type: AlienType::Lordakia,
                position: pos,
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

        // 3. Portals
        // Jump Gate to Sector 1-2
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

        // Jump Gate to Sector 4-4 (PvP)
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
        // 🔴 Red Boss Rift
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

        // 🟢 Green Mineral Rift
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

        // 🟣 Purple Swarm Rift
        self.portals.push(Portal {
            id: self.next_entity_id,
            position: Vec2::new(1350.0, 0.0),
            radius: 65.0,
            portal_type: PortalType::EventRift {
                event_type: EventRiftType::PurpleSwarm,
                time_left: 300.0,
            },
        });
        self.next_entity_id += 1;
    }

    fn try_connect(&mut self, url: &str, token: &str) {
        self.connecting = true;
        self.status_text = format!("Connexion à {}...", url);

        let full_url = if token.is_empty() || token == "guest" {
            format!("{url}?token=guest")
        } else {
            format!("{url}?token={token}")
        };

        net::connect(&full_url);
    }

    fn send_message(&mut self, msg: &ClientMessage) {
        if self.connected {
            if let Ok(bytes) = serialize_packet(msg) {
                net::send(&bytes);
            }
        }
    }

    fn poll_network(&mut self) {
        if !self.connected && net::is_connected() {
            self.connected = true;
            self.connecting = false;
            self.is_offline_sim = false;
            self.status_text = "Connecté au serveur spatial multijoueur".to_string();

            let auth_msg = ClientMessage::Auth {
                token: "guest".to_string(),
            };
            self.send_message(&auth_msg);
        }

        while let Some(bytes) = net::try_recv() {
            if let Ok(server_msg) = deserialize_packet::<ServerMessage>(&bytes) {
                self.handle_server_message(server_msg);
            }
        }
    }

    fn handle_server_message(&mut self, msg: ServerMessage) {
        match msg {
            ServerMessage::AuthSuccess {
                player_id,
                username,
                ship,
                current_map,
            } => {
                self.local_player_id = player_id;
                self.local_ship = ship;
                self.current_map = current_map;
                self.camera_pos = self.local_ship.position;
                self.status_text = format!("Pilote: {}", username);
                self.show_notification(
                    "Connexion Établie",
                    &format!("Bienvenue commandant {}", username),
                    [0.0, 1.0, 0.6, 1.0],
                );
            }
            ServerMessage::AuthError { message } => {
                self.status_text = format!("Erreur: {}", message);
                self.is_offline_sim = true;
            }
            ServerMessage::WorldSnapshot(snapshot) => {
                self.current_map = snapshot.current_map;
                self.lasers = snapshot.lasers;
                self.minerals = snapshot.minerals;
                self.aliens = snapshot.aliens;
                self.loot_boxes = snapshot.loot_boxes;
                self.portals = snapshot.portals;

                for p in snapshot.players {
                    if p.id == self.local_player_id {
                        self.local_ship = p.clone();
                    }
                    self.players.insert(p.id, p);
                }
            }
            ServerMessage::Notification {
                title,
                message,
                color_rgba,
            } => {
                self.show_notification(&title, &message, color_rgba);
            }
            ServerMessage::Pong { client_time, .. } => {
                let now = (macroquad::time::get_time() * 1000.0) as u64;
                if now >= client_time {
                    self.ping_ms = now - client_time;
                }
            }
            _ => {}
        }
    }

    fn show_notification(&mut self, title: &str, msg: &str, _color: [f32; 4]) {
        self.notification_text = format!("📢 {} : {}", title, msg);
        self.notification_timer = 4.5;
    }

    fn spawn_float_text(&mut self, text: String, pos: Vec2, color: Color) {
        self.float_texts.push(FloatText {
            text,
            pos,
            color,
            lifetime: 1.6,
        });
    }

    fn spawn_explosion(&mut self, pos: Vec2, color: Color, count: usize) {
        for i in 0..count {
            let seed = (i as f32 + get_time() as f32) * 23.45;
            let angle = (seed.sin() * 43758.5453).fract() * std::f32::consts::TAU;
            let speed = 40.0 + ((seed + 1.0).sin() * 43758.5453).fract().abs() * 260.0;
            let vel = Vec2::new(angle.cos() * speed, angle.sin() * speed);
            let life = 0.4 + ((seed + 2.0).sin() * 43758.5453).fract().abs() * 0.7;
            let size = 2.0 + ((seed + 3.0).sin() * 43758.5453).fract().abs() * 3.5;

            self.particles.push(Particle {
                pos,
                vel,
                color,
                lifetime: life,
                max_lifetime: life,
                size,
            });
        }
    }

    fn update_fx(&mut self, dt: f32) {
        for p in &mut self.particles {
            p.pos += p.vel * dt;
            p.vel = p.vel * 0.94;
            p.lifetime -= dt;
        }
        self.particles.retain(|p| p.lifetime > 0.0);

        for ft in &mut self.float_texts {
            ft.pos.y -= 28.0 * dt;
            ft.lifetime -= dt;
        }
        self.float_texts.retain(|ft| ft.lifetime > 0.0);

        if self.notification_timer > 0.0 {
            self.notification_timer -= dt;
        }
    }

    // --- Offline Game Loop & Logic ---
    fn update_offline_sim(&mut self, dt: f32) {
        let player_pos = self.local_ship.position;

        // Check Safe Zone status
        let dist_to_base = player_pos.distance_to(self.space_base.position);
        let in_safe_zone = dist_to_base <= self.space_base.radius;
        self.local_ship.is_in_safe_zone = in_safe_zone;

        // Space Base health and shield regeneration
        if in_safe_zone && self.local_ship.is_alive {
            let regen = 25.0 * dt;
            self.local_ship.health = (self.local_ship.health + regen).min(self.local_ship.max_health);
            self.local_ship.shield = (self.local_ship.shield + regen * 1.5).min(self.local_ship.max_shield);
        } else if self.local_ship.is_alive {
            let regen = 4.0 * dt * (1.0 + self.local_ship.talents.defense_regen as f32 * 0.15);
            self.local_ship.shield = (self.local_ship.shield + regen).min(self.local_ship.max_shield);
        }

        // Alien AI Simulation
        let now = get_time() as f32;
        let mut new_alien_lasers = Vec::new();
        let mut dropped_loot = Vec::new();

        for alien in &mut self.aliens {
            if alien.health <= 0.0 {
                continue;
            }

            let dist = alien.position.distance_to(player_pos);
            let aggro_range = alien.alien_type.aggro_range();

            if dist < aggro_range && !in_safe_zone && self.local_ship.is_alive {
                // Aggro player
                let dir = (player_pos - alien.position).normalize();
                alien.rotation = dir.y.atan2(dir.x);
                alien.velocity = dir * alien.alien_type.speed();
                alien.position += alien.velocity * dt;

                // Alien Shoot
                if now - alien.last_shot_time >= alien.alien_type.laser_cooldown() && dist < 420.0 {
                    alien.last_shot_time = now;
                    let laser_dir = dir;
                    let spawn_pos = alien.position + laser_dir * 20.0;
                    new_alien_lasers.push(Laser {
                        id: self.next_entity_id,
                        shooter_id: alien.id,
                        is_alien: true,
                        position: spawn_pos,
                        velocity: laser_dir * (LASER_SPEED * 0.75),
                        lifetime: LASER_LIFETIME,
                        damage: alien.alien_type.laser_damage(),
                        color_rgba: [1.0, 0.2, 0.25, 1.0],
                    });
                    self.next_entity_id += 1;
                }
            } else {
                // Idle wander
                alien.rotation += (alien.id as f32 * 0.1 + now * 0.2).sin() * dt;
                let forward = Vec2::new(alien.rotation.cos(), alien.rotation.sin());
                alien.position += forward * (alien.alien_type.speed() * 0.3 * dt);
            }
        }

        self.lasers.extend(new_alien_lasers);

        // Update Lasers
        for laser in &mut self.lasers {
            laser.position += laser.velocity * dt;
            laser.lifetime -= dt;
        }

        // Laser Hits & Collisions
        let mut hit_alien_ids = Vec::new();
        let mut player_destroyed = false;

        for laser in &mut self.lasers {
            if laser.lifetime <= 0.0 {
                continue;
            }

            if !laser.is_alien {
                // Player laser hits aliens
                for alien in &mut self.aliens {
                    if alien.health > 0.0 && laser.position.distance_to(alien.position) < 26.0 {
                        laser.lifetime = 0.0;
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
                            hit_alien_ids.push(alien.id);
                        }
                        break;
                    }
                }
            } else if !in_safe_zone && self.local_ship.is_alive {
                // Alien laser hits player
                if laser.position.distance_to(player_pos) < SHIP_RADIUS + 4.0 {
                    laser.lifetime = 0.0;
                    let dmg = laser.damage;
                    if self.local_ship.shield > 0.0 {
                        let absorbed = dmg.min(self.local_ship.shield);
                        self.local_ship.shield -= absorbed;
                        let remaining = dmg - absorbed;
                        self.local_ship.health -= remaining;
                    } else {
                        self.local_ship.health -= dmg;
                    }

                    if self.local_ship.health <= 0.0 {
                        self.local_ship.health = 0.0;
                        self.local_ship.is_alive = false;
                        player_destroyed = true;
                    }
                }
            }
        }

        if player_destroyed {
            self.spawn_explosion(player_pos, RED, 40);
            self.spawn_explosion(player_pos, ORANGE, 30);
        }

        self.lasers.retain(|l| l.lifetime > 0.0);

        // Handle Dead Aliens & Drop Loot Boxes
        for dead_id in hit_alien_ids {
            if let Some(pos) = self.aliens.iter().position(|a| a.id == dead_id) {
                let alien = self.aliens.remove(pos);
                self.spawn_explosion(alien.position, ORANGE, 30);
                self.spawn_explosion(alien.position, SKYBLUE, 20);

                let xp = alien.alien_type.xp_reward();
                let credits = alien.alien_type.credits_reward();
                if self.local_ship.add_xp(xp) {
                    self.spawn_float_text(
                        format!("⭐ NIVEAU SUPÉRIEUR ! (Lv. {})", self.local_ship.level),
                        self.local_ship.position + Vec2::new(0.0, -45.0),
                        GOLD,
                    );
                }

                self.local_ship.credits += credits;
                self.spawn_float_text(
                    format!("+{} XP • +{} C.", xp, credits),
                    alien.position + Vec2::new(0.0, -30.0),
                    GREEN,
                );

                // Drop Cargo Box
                dropped_loot.push(LootBox {
                    id: self.next_entity_id,
                    position: alien.position,
                    credits: credits / 2,
                    mineral: Some((MineralType::Prometium, 3)),
                    scrap: 2,
                    plasma_cores: if alien.alien_type == AlienType::Sibelon { 2 } else { 0 },
                    lifetime: 60.0,
                });
                self.next_entity_id += 1;
            }
        }

        self.loot_boxes.extend(dropped_loot);

        // Loot Box Collection
        let mut collected_box_ids = Vec::new();
        let mut collected_notifications = Vec::new();

        for loot in &mut self.loot_boxes {
            loot.lifetime -= dt;
            if loot.position.distance_to(player_pos) < LOOTBOX_RADIUS + SHIP_RADIUS && self.local_ship.is_alive {
                collected_box_ids.push(loot.id);
                self.local_ship.credits += loot.credits;
                self.local_ship.cargo.add_scrap(loot.scrap);
                if loot.plasma_cores > 0 {
                    self.local_ship.cargo.add_plasma_core(loot.plasma_cores);
                }
                if let Some((m, amt)) = loot.mineral {
                    let _ = self.local_ship.cargo.add_mineral(m, amt);
                }

                collected_notifications.push((loot.credits, loot.scrap));
            }
        }

        for (cred, scr) in collected_notifications {
            self.spawn_float_text(
                format!("📦 Fret Récupéré ! (+{} C., +{} Ferraille)", cred, scr),
                player_pos + Vec2::new(0.0, -32.0),
                YELLOW,
            );
        }

        self.loot_boxes.retain(|lb| lb.lifetime > 0.0 && !collected_box_ids.contains(&lb.id));

        // Mining Laser Simulation (Minier Class Only)
        if self.local_ship.is_mining && self.local_ship.is_alive {
            if let Some(target_id) = self.local_ship.mining_target {
                let mut mineral_completed = None;
                let mut spark_needed = false;

                if let Some(mineral) = self.minerals.iter_mut().find(|m| m.id == target_id) {
                    let dist = player_pos.distance_to(mineral.position);
                    if dist <= MINING_RANGE {
                        let talent_speed_bonus = 1.0 + (self.local_ship.talents.logistics_mining_speed as f32 * 0.25);
                        let dmg = MINING_DAMAGE_PER_SEC * talent_speed_bonus * dt;
                        mineral.health -= dmg;
                        spark_needed = true;

                        if mineral.health <= 0.0 {
                            mineral_completed = Some(mineral.mineral_type);
                            mineral.health = mineral.max_health;
                            let seed = get_time() as f32 * 45.6;
                            let angle = (seed.sin() * 43758.5).fract() * std::f32::consts::TAU;
                            let d = 500.0 + ((seed + 1.0).sin() * 43758.5).fract().abs() * 1300.0;
                            mineral.position = Vec2::new(angle.cos() * d, angle.sin() * d);
                        }
                    } else {
                        self.local_ship.is_mining = false;
                        self.local_ship.mining_target = None;
                    }
                } else {
                    self.local_ship.is_mining = false;
                    self.local_ship.mining_target = None;
                }

                if spark_needed && get_time() as f32 - self.mining_sound_timer > 0.08 {
                    self.mining_sound_timer = get_time() as f32;
                    let spark_pos = player_pos + Vec2::new((get_time() as f32 * 20.0).sin() * 8.0, (get_time() as f32 * 30.0).cos() * 8.0);
                    self.particles.push(Particle {
                        pos: spark_pos,
                        vel: Vec2::new((get_time() as f32 * 40.0).sin() * 60.0, -80.0),
                        color: Color::new(1.0, 0.85, 0.2, 1.0),
                        lifetime: 0.25,
                        max_lifetime: 0.25,
                        size: 2.5,
                    });
                }

                if let Some(m_type) = mineral_completed {
                    let added = self.local_ship.cargo.add_mineral(m_type, 1);
                    if added {
                        let xp_gain = m_type.value() * 2;
                        if self.local_ship.add_xp(xp_gain) {
                            self.spawn_float_text(
                                format!("⭐ NIVEAU {} !", self.local_ship.level),
                                player_pos + Vec2::new(0.0, -45.0),
                                GOLD,
                            );
                        }
                        self.spawn_float_text(
                            format!("+1 {} [Soute: {}/{} kg]", m_type.name(), self.local_ship.cargo.used_capacity(), self.local_ship.cargo.max_capacity),
                            player_pos + Vec2::new(0.0, -32.0),
                            Color::new(0.2, 1.0, 0.5, 1.0),
                        );
                    } else {
                        self.spawn_float_text(
                            "⚠️ SOUTE PLEINE ! Vendez à la base".to_string(),
                            player_pos + Vec2::new(0.0, -32.0),
                            RED,
                        );
                    }

                    self.local_ship.is_mining = false;
                    self.local_ship.mining_target = None;
                }
            }
        }
    }

    fn shoot_laser(&mut self) {
        if !self.local_ship.is_alive || self.local_ship.is_in_safe_zone {
            return;
        }

        let dir = Vec2::new(self.local_ship.rotation.cos(), self.local_ship.rotation.sin());
        let spawn_pos = self.local_ship.position + dir * (SHIP_RADIUS + 4.0);

        let talent_dmg_bonus = 1.0 + (self.local_ship.talents.combat_laser_dmg as f32 * 0.06);
        let dmg = self.local_ship.ship_class.base_laser_damage() * talent_dmg_bonus;

        let laser = Laser {
            id: self.next_entity_id,
            shooter_id: self.local_player_id,
            is_alien: false,
            position: spawn_pos,
            velocity: dir * LASER_SPEED + self.local_ship.velocity * 0.3,
            lifetime: LASER_LIFETIME,
            damage: dmg,
            color_rgba: match self.local_ship.ship_class {
                ShipClass::Combat => [0.0, 0.9, 1.0, 1.0],
                ShipClass::Minier => [1.0, 0.75, 0.1, 1.0],
                ShipClass::Transport => [0.2, 0.7, 1.0, 1.0],
                ShipClass::Exploration => [0.2, 1.0, 0.5, 1.0],
            },
        };
        self.next_entity_id += 1;
        self.lasers.push(laser);

        self.send_message(&ClientMessage::Shoot);
    }
}

// --- Main Macroquad Entry Point ---
#[macroquad::main("AstroBrawl")]
async fn main() {
    let mut game = GameClient::new();

    // Try connecting to backend WSS if available
    let default_server_url = "ws://localhost:3000/ws";
    game.try_connect(default_server_url, "guest");

    let mut last_shot_time = 0.0f32;

    loop {
        let dt = get_frame_time().min(0.05);

        // Network polling
        game.poll_network();

        let now = get_time() as f32;
        if (now as f64) - game.last_ping_send > 2.0 && game.connected {
            game.last_ping_send = now as f64;
            let ping = ClientMessage::Ping {
                client_time: (now * 1000.0) as u64,
            };
            game.send_message(&ping);
        }

        // --- Controls & Inputs ---
        let mut thrust = false;
        let mut target_angle = game.local_ship.rotation;
        let mut shoot = false;
        let mut respawn = false;

        // Modal toggle shortcuts
        if is_key_pressed(KeyCode::T) {
            game.active_modal = if game.active_modal == ActiveModal::Talents { ActiveModal::None } else { ActiveModal::Talents };
        }
        if is_key_pressed(KeyCode::H) || is_key_pressed(KeyCode::B) {
            game.active_modal = if game.active_modal == ActiveModal::Hangar { ActiveModal::None } else { ActiveModal::Hangar };
        }
        if is_key_pressed(KeyCode::C) {
            game.active_modal = if game.active_modal == ActiveModal::Crafting { ActiveModal::None } else { ActiveModal::Crafting };
        }

        // Sell Cargo shortcut at space base
        if is_key_pressed(KeyCode::V) && game.local_ship.is_in_safe_zone {
            let credits = game.local_ship.cargo.sell_all_minerals();
            if credits > 0 {
                game.local_ship.credits += credits;
                let pos = game.local_ship.position;
                game.spawn_float_text(format!("💰 Minerais Vendus : +{} C.", credits), pos + Vec2::new(0.0, -35.0), GOLD);
                game.send_message(&ClientMessage::SellCargo);
            }
        }

        if game.local_ship.is_alive && game.active_modal == ActiveModal::None {
            thrust = is_key_down(KeyCode::W)
                || is_key_down(KeyCode::Z)
                || is_key_down(KeyCode::Up)
                || (is_mouse_button_down(MouseButton::Right) && !is_key_down(KeyCode::LeftShift));

            let (mx, my) = mouse_position();
            let screen_center = Vec2::new(screen_width() * 0.5, screen_height() * 0.5);
            let mouse_dir = Vec2::new(mx - screen_center.x, my - screen_center.y);
            target_angle = mouse_dir.y.atan2(mouse_dir.x);

            // Laser Fire
            let talent_rate_bonus = 1.0 + (game.local_ship.talents.combat_fire_rate as f32 * 0.05);
            let cooldown = game.local_ship.ship_class.laser_cooldown() / talent_rate_bonus;
            if (is_mouse_button_down(MouseButton::Left) || is_key_down(KeyCode::Space)) && now - last_shot_time >= cooldown {
                shoot = true;
                last_shot_time = now;
            }

            // Mining Laser Action (Press E or hold when targeting a mineral)
            if is_key_down(KeyCode::E) || (is_mouse_button_down(MouseButton::Right) && is_key_down(KeyCode::LeftShift)) {
                if game.local_ship.ship_class.can_mine() {
                    let p_pos = game.local_ship.position;
                    let mut closest_dist = MINING_RANGE;
                    let mut target_min = None;

                    for m in &game.minerals {
                        let d = p_pos.distance_to(m.position);
                        if d < closest_dist {
                            closest_dist = d;
                            target_min = Some(m.id);
                        }
                    }

                    if let Some(min_id) = target_min {
                        game.local_ship.is_mining = true;
                        game.local_ship.mining_target = Some(min_id);
                        game.send_message(&ClientMessage::StartMining { mineral_id: min_id });
                    }
                } else if is_key_pressed(KeyCode::E) {
                    game.spawn_float_text(
                        "⚠️ Seul l'Extracteur Minier possède un laser de forage !".to_string(),
                        game.local_ship.position + Vec2::new(0.0, -30.0),
                        ORANGE,
                    );
                }
            } else if !is_key_down(KeyCode::E) {
                if game.local_ship.is_mining {
                    game.local_ship.is_mining = false;
                    game.local_ship.mining_target = None;
                    game.send_message(&ClientMessage::StopMining);
                }
            }

            // Jump Portal Action (Press J when near a portal)
            if is_key_pressed(KeyCode::J) {
                let p_pos = game.local_ship.position;
                if let Some(portal) = game.portals.iter().find(|p| p.position.distance_to(p_pos) <= p.radius + 20.0) {
                    match portal.portal_type {
                        PortalType::MapJump { target_map, target_pos } => {
                            game.current_map = target_map;
                            game.local_ship.position = target_pos;
                            game.spawn_float_text(format!("🌀 Saut vers {}", target_map.name()), target_pos, SKYBLUE);
                        }
                        PortalType::EventRift { event_type, .. } => {
                            game.spawn_float_text(format!("🚀 Entrée dans {}", event_type.name()), p_pos, GOLD);
                        }
                    }
                }
            }

            // Thruster particle trail
            if thrust {
                let back_dir = Vec2::new(-game.local_ship.rotation.cos(), -game.local_ship.rotation.sin());
                let exhaust_pos = game.local_ship.position + back_dir * (SHIP_RADIUS + 4.0);

                let thrust_col = match game.local_ship.ship_class {
                    ShipClass::Combat => Color::new(0.1, 0.85, 1.0, 0.9),
                    ShipClass::Minier => Color::new(1.0, 0.65, 0.15, 0.9),
                    ShipClass::Transport => Color::new(0.3, 0.5, 1.0, 0.9),
                    ShipClass::Exploration => Color::new(0.1, 1.0, 0.6, 0.9),
                };

                game.particles.push(Particle {
                    pos: exhaust_pos,
                    vel: back_dir * 130.0 + game.local_ship.velocity * 0.2,
                    color: thrust_col,
                    lifetime: 0.35,
                    max_lifetime: 0.35,
                    size: 3.2,
                });
            }
        } else if !game.local_ship.is_alive {
            if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::R) {
                respawn = true;
            }
        }

        // Apply physics locally
        apply_ship_physics(&mut game.local_ship, thrust, target_angle, dt);

        if respawn {
            game.local_ship.is_alive = true;
            game.local_ship.position = Vec2::ZERO;
            game.local_ship.velocity = Vec2::ZERO;
            game.local_ship.health = game.local_ship.max_health;
            game.local_ship.shield = game.local_ship.max_shield;
            game.send_message(&ClientMessage::Respawn);
        }

        if shoot {
            game.shoot_laser();
        }

        // Run local simulation (or send inputs to server if online)
        if game.is_offline_sim {
            game.update_offline_sim(dt);
        } else {
            let input_msg = ClientMessage::Input {
                thrust,
                target_angle,
            };
            game.send_message(&input_msg);
        }

        // Smooth camera follow
        game.camera_pos = game.camera_pos.lerp(game.local_ship.position, 0.14);
        game.update_fx(dt);

        // ================= RENDERING =================
        clear_background(Color::new(0.015, 0.02, 0.05, 1.0));
        let half_screen = Vec2::new(screen_width() * 0.5, screen_height() * 0.5);

        // 1. Parallax Starfield
        for star in &game.stars {
            let sx = (star.pos.x - game.camera_pos.x * star.layer).rem_euclid(screen_width() + 100.0) - 50.0;
            let sy = (star.pos.y - game.camera_pos.y * star.layer).rem_euclid(screen_height() + 100.0) - 50.0;
            draw_circle(sx, sy, star.size, star.color);
        }

        // 2. Cosmic Grid
        let grid_size = 250.0;
        let start_x = ((game.camera_pos.x - half_screen.x) / grid_size).floor() * grid_size;
        let end_x = ((game.camera_pos.x + half_screen.x) / grid_size).ceil() * grid_size;
        let start_y = ((game.camera_pos.y - half_screen.y) / grid_size).floor() * grid_size;
        let end_y = ((game.camera_pos.y + half_screen.y) / grid_size).ceil() * grid_size;

        let grid_col = Color::new(0.0, 0.5, 0.8, 0.06);
        let mut cur_x = start_x;
        while cur_x <= end_x {
            let sx = cur_x - game.camera_pos.x + half_screen.x;
            draw_line(sx, 0.0, sx, screen_height(), 1.0, grid_col);
            cur_x += grid_size;
        }
        let mut cur_y = start_y;
        while cur_y <= end_y {
            let sy = cur_y - game.camera_pos.y + half_screen.y;
            draw_line(0.0, sy, screen_width(), sy, 1.0, grid_col);
            cur_y += grid_size;
        }

        // 3. Space Base & Safe Zone
        let base_sx = game.space_base.position.x - game.camera_pos.x + half_screen.x;
        let base_sy = game.space_base.position.y - game.camera_pos.y + half_screen.y;

        // Forcefield bubble
        let bubble_pulse = ((now * 2.0).sin() * 0.15 + 0.85).clamp(0.0, 1.0);
        draw_circle_lines(base_sx, base_sy, game.space_base.radius, 2.5, Color::new(0.0, 0.9, 1.0, 0.35 * bubble_pulse));
        draw_circle(base_sx, base_sy, game.space_base.radius, Color::new(0.0, 0.6, 1.0, 0.04));

        // Space Base Station Structure
        draw_circle(base_sx, base_sy, 70.0, Color::new(0.08, 0.12, 0.22, 0.9));
        draw_circle_lines(base_sx, base_sy, 70.0, 3.0, Color::new(0.0, 0.8, 1.0, 0.8));
        draw_circle(base_sx, base_sy, 35.0, Color::new(0.0, 0.5, 0.9, 0.6));
        draw_circle(base_sx, base_sy, 14.0, GOLD);

        let base_name = &game.space_base.name;
        let bn_w = measure_text(base_name, None, 14, 1.0).width;
        draw_text(base_name, base_sx - bn_w * 0.5, base_sy - 85.0, 14.0, SKYBLUE);

        if game.local_ship.is_in_safe_zone {
            let z_text = "🛡️ ZONE DE NON-AGRESSION • SÉCURITÉ ACTIVE [V: Vendre Soute | H: Hangar | C: Craft]";
            let z_w = measure_text(z_text, None, 13, 1.0).width;
            draw_text(z_text, base_sx - z_w * 0.5, base_sy + 95.0, 13.0, GREEN);
        }

        // 4. Portals & Event Rifts
        for portal in &game.portals {
            let px = portal.position.x - game.camera_pos.x + half_screen.x;
            let py = portal.position.y - game.camera_pos.y + half_screen.y;

            let (p_col, p_label) = match &portal.portal_type {
                PortalType::MapJump { target_map, .. } => (Color::new(0.1, 0.7, 1.0, 0.8), target_map.name()),
                PortalType::EventRift { event_type, .. } => {
                    let rgba = event_type.color_rgba();
                    (Color::new(rgba[0], rgba[1], rgba[2], 0.85), event_type.name())
                }
            };

            // Swirling Accretion disk
            let rot = now * 2.5;
            draw_circle(px, py, portal.radius, Color::new(p_col.r, p_col.g, p_col.b, 0.15));
            draw_circle_lines(px, py, portal.radius, 2.5, p_col);

            for i in 0..4 {
                let a = rot + (i as f32) * (std::f32::consts::PI * 0.5);
                let arm_x = px + a.cos() * (portal.radius * 0.7);
                let arm_y = py + a.sin() * (portal.radius * 0.7);
                draw_circle(arm_x, arm_y, 4.0, WHITE);
            }

            let pl_w = measure_text(p_label, None, 13, 1.0).width;
            draw_text(p_label, px - pl_w * 0.5, py - portal.radius - 12.0, 13.0, p_col);

            if game.local_ship.position.distance_to(portal.position) <= portal.radius + 30.0 {
                let jump_hint = "[J] ENTRER DANS LE PORTAIL";
                let jh_w = measure_text(jump_hint, None, 14, 1.0).width;
                draw_text(jump_hint, px - jh_w * 0.5, py + portal.radius + 20.0, 14.0, YELLOW);
            }
        }

        // 5. World Border
        let border_half_w = WORLD_WIDTH * 0.5;
        let border_half_h = WORLD_HEIGHT * 0.5;
        let b_tl_x = -border_half_w - game.camera_pos.x + half_screen.x;
        let b_tl_y = -border_half_h - game.camera_pos.y + half_screen.y;
        let border_glow = ((now * 3.0).sin() * 0.2 + 0.8).clamp(0.0, 1.0);
        draw_rectangle_lines(b_tl_x, b_tl_y, WORLD_WIDTH, WORLD_HEIGHT, 3.0, Color::new(1.0, 0.2, 0.3, 0.6 * border_glow));

        // 6. Minerals
        for mineral in &game.minerals {
            let sx = mineral.position.x - game.camera_pos.x + half_screen.x;
            let sy = mineral.position.y - game.camera_pos.y + half_screen.y;

            if sx >= -50.0 && sx <= screen_width() + 50.0 && sy >= -50.0 && sy <= screen_height() + 50.0 {
                let rgba = mineral.mineral_type.color_rgba();
                let ore_col = Color::new(rgba[0], rgba[1], rgba[2], rgba[3]);

                let pulse = ((now * 4.0 + (mineral.id as f32)).sin() * 0.25 + 0.75).clamp(0.0, 1.0);
                draw_circle(sx, sy, MINERAL_RADIUS * 1.5, Color::new(rgba[0], rgba[1], rgba[2], 0.2 * pulse));

                let r = MINERAL_RADIUS;
                let v1 = to_mq(Vec2::new(sx, sy - r));
                let v2 = to_mq(Vec2::new(sx + r, sy));
                let v3 = to_mq(Vec2::new(sx, sy + r));
                let v4 = to_mq(Vec2::new(sx - r, sy));

                draw_triangle(v1, v2, v3, ore_col);
                draw_triangle(v1, v3, v4, ore_col);
                draw_triangle_lines(v1, v2, v3, 1.5, WHITE);
                draw_triangle_lines(v1, v3, v4, 1.5, WHITE);

                // Health bar if damaged
                if mineral.health < mineral.max_health {
                    let bar_w = 32.0;
                    let bar_h = 3.0;
                    let pct = (mineral.health / mineral.max_health).clamp(0.0, 1.0);
                    draw_rectangle(sx - bar_w * 0.5, sy - 22.0, bar_w, bar_h, DARKGRAY);
                    draw_rectangle(sx - bar_w * 0.5, sy - 22.0, bar_w * pct, bar_h, ore_col);
                }
            }
        }

        // 7. Mining Beam (Rendered between Minier Ship & Mineral)
        if game.local_ship.is_mining && game.local_ship.is_alive {
            if let Some(target_id) = game.local_ship.mining_target {
                if let Some(mineral) = game.minerals.iter().find(|m| m.id == target_id) {
                    let ship_sx = game.local_ship.position.x - game.camera_pos.x + half_screen.x;
                    let ship_sy = game.local_ship.position.y - game.camera_pos.y + half_screen.y;
                    let min_sx = mineral.position.x - game.camera_pos.x + half_screen.x;
                    let min_sy = mineral.position.y - game.camera_pos.y + half_screen.y;

                    // Vibrating extraction beam
                    let beam_pulse = ((now * 45.0).sin() * 2.0).abs();
                    draw_line(ship_sx, ship_sy, min_sx, min_sy, 7.0 + beam_pulse, Color::new(1.0, 0.75, 0.1, 0.45));
                    draw_line(ship_sx, ship_sy, min_sx, min_sy, 2.5, WHITE);
                }
            }
        }

        // 8. Loot Boxes (Boîtes de Fret)
        for loot in &game.loot_boxes {
            let lx = loot.position.x - game.camera_pos.x + half_screen.x;
            let ly = loot.position.y - game.camera_pos.y + half_screen.y;

            let size = 16.0;
            draw_rectangle(lx - size * 0.5, ly - size * 0.5, size, size, Color::new(0.1, 0.6, 0.9, 0.85));
            draw_rectangle_lines(lx - size * 0.5, ly - size * 0.5, size, size, 2.0, GOLD);
            draw_circle(lx, ly, 3.0, WHITE);

            let box_text = "📦 FRET";
            let bw = measure_text(box_text, None, 11, 1.0).width;
            draw_text(box_text, lx - bw * 0.5, ly - 14.0, 11.0, YELLOW);
        }

        // 9. Lasers
        for laser in &game.lasers {
            let sx = laser.position.x - game.camera_pos.x + half_screen.x;
            let sy = laser.position.y - game.camera_pos.y + half_screen.y;

            let dir = laser.velocity.normalize();
            let tail = Vec2::new(sx, sy) - dir * 20.0;
            let head = Vec2::new(sx, sy) + dir * 6.0;

            let col = Color::new(laser.color_rgba[0], laser.color_rgba[1], laser.color_rgba[2], laser.color_rgba[3]);
            draw_line(tail.x, tail.y, head.x, head.y, 6.0, Color::new(col.r, col.g, col.b, 0.4));
            draw_line(tail.x, tail.y, head.x, head.y, 2.5, WHITE);
        }

        // 10. Aliens
        for alien in &game.aliens {
            if alien.health <= 0.0 {
                continue;
            }

            let ax = alien.position.x - game.camera_pos.x + half_screen.x;
            let ay = alien.position.y - game.camera_pos.y + half_screen.y;

            let rot = alien.rotation;
            let (nose_len, side_len, alien_col) = match alien.alien_type {
                AlienType::Streuner => (20.0, 16.0, RED),
                AlienType::Lordakia => (26.0, 22.0, ORANGE),
                AlienType::Sibelon => (44.0, 40.0, PURPLE),
            };

            let nose = to_mq(Vec2::new(ax + rot.cos() * nose_len, ay + rot.sin() * nose_len));
            let left = to_mq(Vec2::new(ax + (rot + 2.4).cos() * side_len, ay + (rot + 2.4).sin() * side_len));
            let right = to_mq(Vec2::new(ax + (rot - 2.4).cos() * side_len, ay + (rot - 2.4).sin() * side_len));

            draw_triangle(nose, left, right, Color::new(0.2, 0.05, 0.08, 1.0));
            draw_triangle_lines(nose, left, right, 2.0, alien_col);

            // Alien HP Bar
            let bar_w = side_len * 2.0;
            let bar_h = 3.5;
            let hp_pct = (alien.health / alien.max_health).clamp(0.0, 1.0);
            draw_rectangle(ax - bar_w * 0.5, ay - side_len - 14.0, bar_w, bar_h, DARKGRAY);
            draw_rectangle(ax - bar_w * 0.5, ay - side_len - 14.0, bar_w * hp_pct, bar_h, RED);

            let a_name = alien.alien_type.name();
            let an_w = measure_text(a_name, None, 12, 1.0).width;
            draw_text(a_name, ax - an_w * 0.5, ay - side_len - 18.0, 12.0, WHITE);
        }

        // 11. Other Multiplayer Players
        for player in game.players.values() {
            if player.id == game.local_player_id || !player.is_alive {
                continue;
            }

            let px = player.position.x - game.camera_pos.x + half_screen.x;
            let py = player.position.y - game.camera_pos.y + half_screen.y;
            let rot = player.rotation;

            let nose = to_mq(Vec2::new(px + rot.cos() * 24.0, py + rot.sin() * 24.0));
            let left_wing = to_mq(Vec2::new(px + (rot + 2.5).cos() * 20.0, py + (rot + 2.5).sin() * 20.0));
            let right_wing = to_mq(Vec2::new(px + (rot - 2.5).cos() * 20.0, py + (rot - 2.5).sin() * 20.0));
            let engine = to_mq(Vec2::new(px + (rot + std::f32::consts::PI).cos() * 14.0, py + (rot + std::f32::consts::PI).sin() * 14.0));

            draw_triangle(nose, left_wing, engine, Color::new(0.65, 0.15, 0.2, 1.0));
            draw_triangle(nose, right_wing, engine, Color::new(0.65, 0.15, 0.2, 1.0));
            draw_triangle_lines(nose, left_wing, engine, 2.0, RED);
            draw_triangle_lines(nose, right_wing, engine, 2.0, RED);

            let name_w = measure_text(&player.username, None, 13, 1.0).width;
            draw_text(&player.username, px - name_w * 0.5, py - 32.0, 13.0, WHITE);
        }

        // 12. Local Player Ship (Always Rendered!)
        if game.local_ship.is_alive {
            let sx = game.local_ship.position.x - game.camera_pos.x + half_screen.x;
            let sy = game.local_ship.position.y - game.camera_pos.y + half_screen.y;
            let rot = game.local_ship.rotation;

            let nose = to_mq(Vec2::new(sx + rot.cos() * 25.0, sy + rot.sin() * 25.0));
            let left_wing = to_mq(Vec2::new(sx + (rot + 2.45).cos() * 22.0, sy + (rot + 2.45).sin() * 22.0));
            let right_wing = to_mq(Vec2::new(sx + (rot - 2.45).cos() * 22.0, sy + (rot - 2.45).sin() * 22.0));
            let engine = to_mq(Vec2::new(sx + (rot + std::f32::consts::PI).cos() * 15.0, sy + (rot + std::f32::consts::PI).sin() * 15.0));

            // Thruster flame
            if game.local_ship.is_thrusting {
                let flame_len = 16.0 + ((now * 32.0).sin() * 4.0);
                let flame_tip = to_mq(Vec2::new(sx + (rot + std::f32::consts::PI).cos() * (15.0 + flame_len), sy + (rot + std::f32::consts::PI).sin() * (15.0 + flame_len)));
                draw_triangle(left_wing * 0.4 + engine * 0.6, flame_tip, right_wing * 0.4 + engine * 0.6, ORANGE);
            }

            let (hull_col, border_col) = match game.local_ship.ship_class {
                ShipClass::Combat => (Color::new(0.06, 0.45, 0.75, 1.0), SKYBLUE),
                ShipClass::Minier => (Color::new(0.75, 0.45, 0.08, 1.0), GOLD),
                ShipClass::Transport => (Color::new(0.2, 0.35, 0.6, 1.0), BLUE),
                ShipClass::Exploration => (Color::new(0.08, 0.6, 0.35, 1.0), GREEN),
            };

            draw_triangle(nose, left_wing, engine, hull_col);
            draw_triangle(nose, right_wing, engine, hull_col);
            draw_triangle_lines(nose, left_wing, engine, 2.0, border_col);
            draw_triangle_lines(nose, right_wing, engine, 2.0, border_col);

            // Cockpit glass
            let cockpit = Vec2::new(sx + rot.cos() * 7.0, sy + rot.sin() * 7.0);
            draw_circle(cockpit.x, cockpit.y, 4.2, Color::new(0.2, 0.95, 1.0, 0.95));

            // Mining pods if Minier
            if game.local_ship.ship_class == ShipClass::Minier {
                let pod1 = Vec2::new(sx + (rot + 1.6).cos() * 16.0, sy + (rot + 1.6).sin() * 16.0);
                let pod2 = Vec2::new(sx + (rot - 1.6).cos() * 16.0, sy + (rot - 1.6).sin() * 16.0);
                draw_circle(pod1.x, pod1.y, 4.0, GOLD);
                draw_circle(pod2.x, pod2.y, 4.0, GOLD);
            }

            // Player Name & Title
            let p_name = format!("{} [{}]", game.local_ship.username, game.local_ship.ship_class.name());
            let nw = measure_text(&p_name, None, 13, 1.0).width;
            draw_text(&p_name, sx - nw * 0.5, sy - 36.0, 13.0, GOLD);

            // Shield & HP Bars
            let bar_w = 48.0;
            let bar_h = 4.0;
            let bar_x = sx - bar_w * 0.5;

            let shield_pct = (game.local_ship.shield / game.local_ship.max_shield).clamp(0.0, 1.0);
            draw_rectangle(bar_x, sy - 30.0, bar_w, bar_h, Color::new(0.05, 0.1, 0.2, 0.8));
            draw_rectangle(bar_x, sy - 30.0, bar_w * shield_pct, bar_h, Color::new(0.0, 0.85, 1.0, 1.0));

            let hp_pct = (game.local_ship.health / game.local_ship.max_health).clamp(0.0, 1.0);
            draw_rectangle(bar_x, sy - 24.0, bar_w, bar_h, Color::new(0.2, 0.05, 0.05, 0.8));
            draw_rectangle(bar_x, sy - 24.0, bar_w * hp_pct, bar_h, Color::new(0.1, 0.95, 0.3, 1.0));
        }

        // 13. Particles & Floating Texts
        for p in &game.particles {
            let sx = p.pos.x - game.camera_pos.x + half_screen.x;
            let sy = p.pos.y - game.camera_pos.y + half_screen.y;
            let alpha = (p.lifetime / p.max_lifetime).clamp(0.0, 1.0);
            let mut col = p.color;
            col.a *= alpha;
            draw_circle(sx, sy, p.size * alpha, col);
        }

        for ft in &game.float_texts {
            let sx = ft.pos.x - game.camera_pos.x + half_screen.x;
            let sy = ft.pos.y - game.camera_pos.y + half_screen.y;
            let mut col = ft.color;
            col.a = (ft.lifetime / 1.6).clamp(0.0, 1.0);
            draw_text(&ft.text, sx, sy, 15.0, col);
        }

        // 14. Top Sci-Fi HUD
        let hud_w = 780.0_f32.min(screen_width() - 40.0);
        let hud_h = 44.0;
        let hud_x = (screen_width() - hud_w) * 0.5;
        let hud_y = 12.0;

        draw_rectangle(hud_x, hud_y, hud_w, hud_h, Color::new(0.04, 0.07, 0.14, 0.92));
        draw_rectangle_lines(hud_x, hud_y, hud_w, hud_h, 1.5, Color::new(0.0, 0.9, 1.0, 0.4));

        let credits_str = format!("💰 Crédits: {}", game.local_ship.credits);
        draw_text(&credits_str, hud_x + 16.0, hud_y + 27.0, 14.0, GOLD);

        let cargo_used = game.local_ship.cargo.used_capacity();
        let cargo_max = game.local_ship.cargo.max_capacity;
        let cargo_col = if game.local_ship.cargo.is_full() { RED } else { SKYBLUE };
        let cargo_str = format!("📦 Soute: {} / {} kg", cargo_used, cargo_max);
        draw_text(&cargo_str, hud_x + 175.0, hud_y + 27.0, 14.0, cargo_col);

        let level_str = format!("⭐ Lv. {} ({} / {} XP)", game.local_ship.level, game.local_ship.xp, game.local_ship.next_level_xp);
        draw_text(&level_str, hud_x + 355.0, hud_y + 27.0, 14.0, Color::new(0.2, 1.0, 0.5, 1.0));

        let map_str = format!("🌐 {}", game.current_map.name());
        draw_text(&map_str, hud_x + 555.0, hud_y + 27.0, 13.0, Color::new(0.7, 0.85, 1.0, 0.9));

        // 15. Notification Banner
        if game.notification_timer > 0.0 {
            let nw = measure_text(&game.notification_text, None, 14, 1.0).width;
            let nx = (screen_width() - nw - 40.0) * 0.5;
            draw_rectangle(nx, 64.0, nw + 40.0, 30.0, Color::new(0.05, 0.1, 0.2, 0.9));
            draw_rectangle_lines(nx, 64.0, nw + 40.0, 30.0, 1.5, SKYBLUE);
            draw_text(&game.notification_text, nx + 20.0, 84.0, 14.0, WHITE);
        }

        // 16. Radar Minimap
        let radar_size = 145.0;
        let rx = screen_width() - radar_size - 18.0;
        let ry = screen_height() - radar_size - 18.0;

        draw_rectangle(rx, ry, radar_size, radar_size, Color::new(0.03, 0.06, 0.12, 0.88));
        draw_rectangle_lines(rx, ry, radar_size, radar_size, 1.5, Color::new(0.0, 0.9, 1.0, 0.5));

        let radar_scale = radar_size / WORLD_WIDTH;
        let radar_center = Vec2::new(rx + radar_size * 0.5, ry + radar_size * 0.5);

        // Safe Base on Radar
        draw_circle_lines(radar_center.x, radar_center.y, game.space_base.radius * radar_scale, 1.0, Color::new(0.0, 0.8, 1.0, 0.4));

        // Minerals on Radar
        for m in &game.minerals {
            let mx = radar_center.x + m.position.x * radar_scale;
            let my = radar_center.y + m.position.y * radar_scale;
            draw_circle(mx, my, 1.2, Color::new(0.2, 0.9, 1.0, 0.6));
        }

        // Aliens on Radar
        for a in &game.aliens {
            if a.health > 0.0 {
                let ax = radar_center.x + a.position.x * radar_scale;
                let ay = radar_center.y + a.position.y * radar_scale;
                draw_circle(ax, ay, 2.0, RED);
            }
        }

        // Loot Boxes on Radar
        for lb in &game.loot_boxes {
            let lx = radar_center.x + lb.position.x * radar_scale;
            let ly = radar_center.y + lb.position.y * radar_scale;
            draw_circle(lx, ly, 2.2, GOLD);
        }

        // Player on Radar
        let px = radar_center.x + game.local_ship.position.x * radar_scale;
        let py = radar_center.y + game.local_ship.position.y * radar_scale;
        draw_circle(px, py, 3.2, GREEN);

        // Radar Sweep Line
        let sweep_a = now * 2.2;
        draw_line(radar_center.x, radar_center.y, radar_center.x + sweep_a.cos() * (radar_size * 0.5), radar_center.y + sweep_a.sin() * (radar_size * 0.5), 1.0, Color::new(0.0, 1.0, 0.8, 0.3));

        // 17. Quick Commands Help & Shortcuts
        draw_rectangle(16.0, screen_height() - 110.0, 340.0, 94.0, Color::new(0.04, 0.07, 0.12, 0.85));
        draw_rectangle_lines(16.0, screen_height() - 110.0, 340.0, 94.0, 1.0, Color::new(0.0, 0.9, 1.0, 0.3));
        draw_text("⚡ ASTROBRAWL RACCOURCIS :", 26.0, screen_height() - 92.0, 12.0, SKYBLUE);
        draw_text("• [Z/W / Clic Droit] Propulseur  • [Espace / Clic Gauche] Tirer", 26.0, screen_height() - 76.0, 11.0, WHITE);
        draw_text("• [E] Laser Minier (Classe Minier uniquement)", 26.0, screen_height() - 60.0, 11.0, GOLD);
        draw_text("• [H] Hangar  • [T] Arbre Talents  • [C] Craft  • [V] Vente Soute", 26.0, screen_height() - 44.0, 11.0, GREEN);
        draw_text("• [J] Sauter dans un Portail / Faille cosmique", 26.0, screen_height() - 28.0, 11.0, SKYBLUE);

        // 18. Modals (Hangar, Talents, Crafting)
        match game.active_modal {
            ActiveModal::Hangar => {
                let mw = 520.0;
                let mh = 360.0;
                let mx = (screen_width() - mw) * 0.5;
                let my = (screen_height() - mh) * 0.5;

                draw_rectangle(mx, my, mw, mh, Color::new(0.05, 0.08, 0.16, 0.96));
                draw_rectangle_lines(mx, my, mw, mh, 2.0, SKYBLUE);
                draw_text("🚀 HANGAR DE SÉLECTION DU VAISSEAU", mx + 24.0, my + 38.0, 20.0, GOLD);
                draw_text("Appuyez sur 1, 2, 3 ou 4 pour changer de classe :", mx + 24.0, my + 65.0, 13.0, WHITE);

                let classes = [
                    (KeyCode::Key1, ShipClass::Combat, "1. Intercepteur de Combat", "Canons surchargés (+32 dmg), grande agilité. Ne peut pas miner."),
                    (KeyCode::Key2, ShipClass::Minier, "2. Extracteur Minier", "Équipé du Laser de Forage thermique. Seul capable de miner ! (+300 kg soute)"),
                    (KeyCode::Key3, ShipClass::Transport, "3. Mastodonte Cargo", "Blindage massif (+260 HP, +240 bouclier), soute titanesque (1000 kg)."),
                    (KeyCode::Key4, ShipClass::Exploration, "4. Éclaireur Longue Portée", "Vitesse suprême (480 px/s), radar étendu pour détecter les failles."),
                ];

                for (idx, (key, class, title, desc)) in classes.iter().enumerate() {
                    let card_y = my + 90.0 + (idx as f32 * 60.0);
                    let is_active = game.local_ship.ship_class == *class;
                    let bg_col = if is_active { Color::new(0.1, 0.3, 0.5, 0.8) } else { Color::new(0.08, 0.12, 0.22, 0.6) };
                    draw_rectangle(mx + 20.0, card_y, mw - 40.0, 52.0, bg_col);
                    draw_rectangle_lines(mx + 20.0, card_y, mw - 40.0, 52.0, 1.0, if is_active { GOLD } else { DARKGRAY });
                    draw_text(title, mx + 32.0, card_y + 22.0, 14.0, if is_active { GOLD } else { WHITE });
                    draw_text(desc, mx + 32.0, card_y + 40.0, 11.0, GRAY);

                    if is_key_pressed(*key) {
                        game.local_ship.ship_class = *class;
                        game.local_ship.max_health = class.base_health();
                        game.local_ship.health = class.base_health();
                        game.local_ship.max_shield = class.base_shield();
                        game.local_ship.shield = class.base_shield();
                        game.local_ship.apply_talent_bonuses();
                        game.spawn_float_text(format!("🚀 Vaisseau équipé : {}", class.name()), game.local_ship.position, SKYBLUE);
                        game.send_message(&ClientMessage::SelectClass { class: *class });
                    }
                }

                draw_text("[H ou Échap pour fermer le hangar]", mx + 130.0, my + mh - 16.0, 12.0, SKYBLUE);
            }
            ActiveModal::Talents => {
                let mw = 560.0;
                let mh = 380.0;
                let mx = (screen_width() - mw) * 0.5;
                let my = (screen_height() - mh) * 0.5;

                draw_rectangle(mx, my, mw, mh, Color::new(0.05, 0.08, 0.16, 0.96));
                draw_rectangle_lines(mx, my, mw, mh, 2.0, Color::new(0.2, 1.0, 0.5, 1.0));
                draw_text("🧬 ARBRE DE TALENTS DU PILOTE", mx + 24.0, my + 38.0, 20.0, Color::new(0.2, 1.0, 0.5, 1.0));
                draw_text(&format!("Points de compétence disponibles : {}", game.local_ship.talent_points), mx + 24.0, my + 65.0, 14.0, GOLD);

                let talents_data = [
                    (KeyCode::Key1, 0, "1. Optique Laser (+6% Dégâts)", game.local_ship.talents.combat_laser_dmg),
                    (KeyCode::Key2, 1, "2. Surchauffeur (+5% Cadence)", game.local_ship.talents.combat_fire_rate),
                    (KeyCode::Key3, 2, "3. Condensateur Bouclier (+10% Bouclier Max)", game.local_ship.talents.defense_shield_max),
                    (KeyCode::Key4, 3, "4. Nano-Réparateur (+15% Régénération)", game.local_ship.talents.defense_regen),
                    (KeyCode::Key5, 4, "5. Soute Compressée (+20% Capacité Soute)", game.local_ship.talents.logistics_cargo),
                    (KeyCode::Key6, 5, "6. Foreuse Thermique (+25% Vitesse Minage)", game.local_ship.talents.logistics_mining_speed),
                ];

                for (idx, (key, talent_idx, title, level)) in talents_data.iter().enumerate() {
                    let ty = my + 90.0 + (idx as f32 * 42.0);
                    draw_rectangle(mx + 20.0, ty, mw - 40.0, 36.0, Color::new(0.08, 0.12, 0.22, 0.7));
                    draw_rectangle_lines(mx + 20.0, ty, mw - 40.0, 36.0, 1.0, DARKGRAY);

                    draw_text(title, mx + 32.0, ty + 23.0, 13.0, WHITE);
                    draw_text(&format!("Niveau : {} / 5", level), mx + 380.0, ty + 23.0, 13.0, SKYBLUE);
                    draw_text("[+]", mx + mw - 60.0, ty + 23.0, 13.0, GREEN);

                    if is_key_pressed(*key) && game.local_ship.talent_points > 0 && *level < 5 {
                        game.local_ship.talent_points -= 1;
                        match talent_idx {
                            0 => game.local_ship.talents.combat_laser_dmg += 1,
                            1 => game.local_ship.talents.combat_fire_rate += 1,
                            2 => game.local_ship.talents.defense_shield_max += 1,
                            3 => game.local_ship.talents.defense_regen += 1,
                            4 => game.local_ship.talents.logistics_cargo += 1,
                            5 => game.local_ship.talents.logistics_mining_speed += 1,
                            _ => {}
                        }
                        game.local_ship.apply_talent_bonuses();
                        game.send_message(&ClientMessage::UpgradeTalent { talent_index: *talent_idx });
                    }
                }

                draw_text("[T ou Échap pour fermer l'arbre]", mx + 160.0, my + mh - 16.0, 12.0, SKYBLUE);
            }
            ActiveModal::Crafting => {
                let mw = 580.0;
                let mh = 390.0;
                let mx = (screen_width() - mw) * 0.5;
                let my = (screen_height() - mh) * 0.5;

                draw_rectangle(mx, my, mw, mh, Color::new(0.05, 0.08, 0.16, 0.96));
                draw_rectangle_lines(mx, my, mw, mh, 2.0, YELLOW);
                draw_text("🛠️ ATELIER D'ASSEMBLAGE (CRAFT)", mx + 24.0, my + 38.0, 20.0, YELLOW);
                draw_text(&format!("Ressources : {} Prometium | {} Endurium | {} Terbium | {} Ferraille",
                    game.local_ship.cargo.prometium, game.local_ship.cargo.endurium, game.local_ship.cargo.terbium, game.local_ship.cargo.scrap),
                    mx + 24.0, my + 64.0, 12.0, SKYBLUE);

                let keys = [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Key4];
                for (idx, recipe) in CRAFT_RECIPES.iter().enumerate() {
                    let ry = my + 86.0 + (idx as f32 * 66.0);
                    let can_craft = game.local_ship.cargo.prometium >= recipe.cost_prometium
                        && game.local_ship.cargo.endurium >= recipe.cost_endurium
                        && game.local_ship.cargo.terbium >= recipe.cost_terbium
                        && game.local_ship.cargo.scrap >= recipe.cost_scrap
                        && game.local_ship.credits >= recipe.cost_credits;

                    draw_rectangle(mx + 20.0, ry, mw - 40.0, 56.0, Color::new(0.08, 0.12, 0.22, 0.7));
                    draw_rectangle_lines(mx + 20.0, ry, mw - 40.0, 56.0, 1.0, if can_craft { GREEN } else { DARKGRAY });

                    draw_text(&format!("{}. {}", idx + 1, recipe.name), mx + 30.0, ry + 20.0, 14.0, if can_craft { GOLD } else { WHITE });
                    draw_text(recipe.description, mx + 30.0, ry + 36.0, 11.0, GRAY);
                    draw_text(&format!("Coût: {} Prom., {} End., {} Terb., {} Ferraille, {} C.",
                        recipe.cost_prometium, recipe.cost_endurium, recipe.cost_terbium, recipe.cost_scrap, recipe.cost_credits),
                        mx + 30.0, ry + 50.0, 10.0, if can_craft { GREEN } else { RED });

                    if is_key_pressed(keys[idx]) && can_craft {
                        game.local_ship.cargo.prometium -= recipe.cost_prometium;
                        game.local_ship.cargo.endurium -= recipe.cost_endurium;
                        game.local_ship.cargo.terbium -= recipe.cost_terbium;
                        game.local_ship.cargo.scrap -= recipe.cost_scrap;
                        game.local_ship.credits -= recipe.cost_credits;

                        if recipe.id == "shield_booster" {
                            game.local_ship.max_shield += 20.0;
                            game.local_ship.shield += 20.0;
                        } else if recipe.id == "cargo_extender" {
                            game.local_ship.cargo.max_capacity += 40;
                        }

                        game.spawn_float_text(format!("✨ Assemblé avec succès : {}", recipe.name), game.local_ship.position, GOLD);
                        game.send_message(&ClientMessage::Craft { recipe_index: idx as u32 });
                    }
                }

                draw_text("[C ou Échap pour fermer l'atelier]", mx + 160.0, my + mh - 16.0, 12.0, SKYBLUE);
            }
            ActiveModal::None => {}
        }

        // 19. Respawn Overlay
        if !game.local_ship.is_alive {
            draw_rectangle(0.0, screen_height() * 0.35, screen_width(), 130.0, Color::new(0.05, 0.05, 0.1, 0.9));
            let d_title = "⚡ VAISSEAU DÉTRUIT ⚡";
            let dw = measure_text(d_title, None, 32, 1.0).width;
            draw_text(d_title, (screen_width() - dw) * 0.5, screen_height() * 0.43, 32.0, RED);

            let sub = "Appuyez sur ESPACE ou R pour vous réincarner à la Base Spatiale";
            let sw = measure_text(sub, None, 17, 1.0).width;
            draw_text(sub, (screen_width() - sw) * 0.5, screen_height() * 0.49, 17.0, WHITE);
        }

        next_frame().await;
    }
}
