use astrobrawl_shared::Vec2;
use astrobrawl_shared::*;
use macroquad::prelude::*;
use std::collections::HashMap;

fn to_mq(v: Vec2) -> macroquad::math::Vec2 {
    vec2(v.x, v.y)
}

pub mod mocha {
    use macroquad::color::Color;

    pub const ROSEWATER: Color = Color::new(0.96, 0.88, 0.86, 1.0);
    pub const FLAMINGO: Color  = Color::new(0.95, 0.80, 0.80, 1.0);
    pub const PINK: Color      = Color::new(0.96, 0.76, 0.91, 1.0);
    pub const MAUVE: Color     = Color::new(0.80, 0.65, 0.97, 1.0);
    pub const RED: Color       = Color::new(0.95, 0.55, 0.66, 1.0);
    pub const MAROON: Color    = Color::new(0.92, 0.63, 0.67, 1.0);
    pub const PEACH: Color     = Color::new(0.98, 0.70, 0.53, 1.0);
    pub const YELLOW: Color    = Color::new(0.98, 0.89, 0.69, 1.0);
    pub const GREEN: Color     = Color::new(0.65, 0.89, 0.63, 1.0);
    pub const TEAL: Color      = Color::new(0.58, 0.89, 0.84, 1.0);
    pub const SKY: Color       = Color::new(0.54, 0.86, 0.92, 1.0);
    pub const SAPPHIRE: Color  = Color::new(0.45, 0.78, 0.93, 1.0);
    pub const BLUE: Color      = Color::new(0.54, 0.71, 0.98, 1.0);
    pub const LAVENDER: Color  = Color::new(0.71, 0.75, 0.99, 1.0);
    pub const TEXT: Color      = Color::new(0.80, 0.84, 0.96, 1.0);
    pub const SUBTEXT1: Color  = Color::new(0.73, 0.76, 0.87, 1.0);
    pub const SUBTEXT0: Color  = Color::new(0.65, 0.68, 0.78, 1.0);
    pub const OVERLAY2: Color  = Color::new(0.58, 0.60, 0.70, 1.0);
    pub const OVERLAY1: Color  = Color::new(0.50, 0.52, 0.61, 1.0);
    pub const OVERLAY0: Color  = Color::new(0.42, 0.44, 0.53, 1.0);
    pub const SURFACE2: Color  = Color::new(0.35, 0.36, 0.44, 1.0);
    pub const SURFACE1: Color  = Color::new(0.27, 0.28, 0.35, 1.0);
    pub const SURFACE0: Color  = Color::new(0.19, 0.20, 0.27, 1.0);
    pub const BASE: Color      = Color::new(0.12, 0.12, 0.18, 1.0);
    pub const MANTLE: Color    = Color::new(0.09, 0.09, 0.15, 1.0);
    pub const CRUST: Color     = Color::new(0.07, 0.07, 0.11, 1.0);
}

fn draw_crisp_text(font: Option<&Font>, text: &str, x: f32, y: f32, size: f32, color: Color) {
    if let Some(f) = font {
        draw_text_ex(
            text,
            x,
            y,
            TextParams {
                font: Some(f),
                font_size: size as u16,
                font_scale: 1.0,
                font_scale_aspect: 1.0,
                rotation: 0.0,
                color,
            },
        );
    } else {
        draw_text(text, x, y, size, color);
    }
}

fn draw_crisp_text_shadow(font: Option<&Font>, text: &str, x: f32, y: f32, size: f32, color: Color) {
    draw_crisp_text(font, text, x + 1.2, y + 1.2, size, Color::new(0.07, 0.07, 0.11, 0.85));
    draw_crisp_text(font, text, x, y, size, color);
}

fn measure_crisp_text(font: Option<&Font>, text: &str, size: f32) -> TextDimensions {
    measure_text(text, font, size as u16, 1.0)
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

// --- Audio Engine Bridge ---
pub mod audio {
    pub const SFX_LASER: i32 = 1;
    pub const SFX_MINING: i32 = 2;
    pub const SFX_EXPLOSION: i32 = 3;
    pub const SFX_TARGET_LOCK: i32 = 4;
    pub const SFX_TARGET_LOST: i32 = 5;
    pub const SFX_COLLECT: i32 = 6;
    pub const SFX_LEVEL_UP: i32 = 7;
    pub const SFX_WARP: i32 = 8;
    pub const SFX_SHIELD_HIT: i32 = 9;
    pub const SFX_HULL_HIT: i32 = 10;
    pub const SFX_CRAFT: i32 = 11;
    pub const SFX_UI_CLICK: i32 = 12;
    pub const SFX_ZONE_SAFE: i32 = 13;
    pub const SFX_RESPAWN: i32 = 14;
    pub const SFX_ALARM: i32 = 15;

    #[cfg(target_arch = "wasm32")]
    extern "C" {
        pub fn mq_play_sfx(id: i32);
        pub fn mq_toggle_mute() -> i32;
    }

    #[cfg(target_arch = "wasm32")]
    pub fn play(id: i32) {
        unsafe { mq_play_sfx(id) };
    }

    #[cfg(target_arch = "wasm32")]
    pub fn toggle_mute() -> bool {
        unsafe { mq_toggle_mute() == 1 }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn play(_id: i32) {}

    #[cfg(not(target_arch = "wasm32"))]
    pub fn toggle_mute() -> bool {
        false
    }
}

// --- Local Storage Persistence Bridge ---
pub mod storage {
    #[cfg(target_arch = "wasm32")]
    extern "C" {
        pub fn mq_save_progression(ptr: *const u8, len: usize);
        pub fn mq_load_progression(dest_ptr: *mut u8, max_len: usize) -> i32;
        pub fn mq_get_player_name(dest_ptr: *mut u8, max_len: usize) -> i32;
        pub fn mq_get_server_url(dest_ptr: *mut u8, max_len: usize) -> i32;
        pub fn mq_get_auth_token(dest_ptr: *mut u8, max_len: usize) -> i32;
        pub fn mq_get_guest_id(dest_ptr: *mut u8, max_len: usize) -> i32;
        pub fn mq_set_connection_status(status: i32);
    }

    #[cfg(target_arch = "wasm32")]
    pub fn save(bytes: &[u8]) {
        unsafe { mq_save_progression(bytes.as_ptr(), bytes.len()) };
    }

    #[cfg(target_arch = "wasm32")]
    pub fn load() -> Option<Vec<u8>> {
        let mut buf = vec![0u8; 16384];
        let len = unsafe { mq_load_progression(buf.as_mut_ptr(), buf.len()) };
        if len > 0 {
            buf.truncate(len as usize);
            Some(buf)
        } else {
            None
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn get_player_name() -> String {
        let mut buf = vec![0u8; 256];
        let len = unsafe { mq_get_player_name(buf.as_mut_ptr(), buf.len()) };
        if len > 0 {
            buf.truncate(len as usize);
            String::from_utf8(buf).unwrap_or_default().trim().to_string()
        } else {
            String::new()
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn get_server_url() -> String {
        let mut buf = vec![0u8; 512];
        let len = unsafe { mq_get_server_url(buf.as_mut_ptr(), buf.len()) };
        if len > 0 {
            buf.truncate(len as usize);
            if let Ok(s) = String::from_utf8(buf) {
                let trimmed = s.trim();
                if !trimmed.is_empty() {
                    return trimmed.to_string();
                }
            }
        }
        "wss://astrobrawl-server.onrender.com/ws".to_string()
    }

    #[cfg(target_arch = "wasm32")]
    pub fn get_auth_token() -> String {
        let mut buf = vec![0u8; 512];
        let len = unsafe { mq_get_auth_token(buf.as_mut_ptr(), buf.len()) };
        if len > 0 {
            buf.truncate(len as usize);
            if let Ok(s) = String::from_utf8(buf) {
                let trimmed = s.trim();
                if !trimmed.is_empty() {
                    return trimmed.to_string();
                }
            }
        }
        "guest".to_string()
    }

    #[cfg(target_arch = "wasm32")]
    pub fn get_guest_id() -> String {
        let mut buf = vec![0u8; 512];
        let len = unsafe { mq_get_guest_id(buf.as_mut_ptr(), buf.len()) };
        if len > 0 {
            buf.truncate(len as usize);
            if let Ok(s) = String::from_utf8(buf) {
                let trimmed = s.trim();
                if !trimmed.is_empty() {
                    return trimmed.to_string();
                }
            }
        }
        String::new()
    }

    #[cfg(target_arch = "wasm32")]
    pub fn set_connection_status(status: i32) {
        unsafe { mq_set_connection_status(status) };
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn save(_bytes: &[u8]) {}

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load() -> Option<Vec<u8>> {
        None
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_player_name() -> String {
        "Chomiam".to_string()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_server_url() -> String {
        "ws://localhost:3000/ws".to_string()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_auth_token() -> String {
        "guest".to_string()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_guest_id() -> String {
        "desktop_dev".to_string()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_connection_status(_status: i32) {}
}

// PlayerSaveData is defined in astrobrawl_shared

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
    locked_target_id: Option<u64>,
    last_ping_send: f64,
    last_reconnect_attempt: f64,
    ping_ms: u64,
    active_modal: ActiveModal,
    notification_text: String,
    notification_timer: f32,
    is_muted: bool,
    alarm_sound_timer: f32,
    autosave_timer: f32,

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
            status_text: "Recherche du serveur spatial...".to_string(),

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
            locked_target_id: None,
            last_ping_send: 0.0,
            last_reconnect_attempt: 0.0,
            ping_ms: 0,
            active_modal: ActiveModal::None,
            notification_text: "Bienvenue dans AstroBrawl ! Rejoignez la base spatiale au centre.".to_string(),
            notification_timer: 6.0,
            is_muted: false,
            alarm_sound_timer: 0.0,
            autosave_timer: 0.0,

            particles: Vec::new(),
            float_texts: Vec::new(),
            stars,
        };

        // Load perpetual progression from localStorage
        client.load_progression();
        client
    }


    pub fn load_progression(&mut self) {
        let stored_name = storage::get_player_name();
        if !stored_name.is_empty() {
            self.local_ship.username = stored_name;
        }

        if let Some(bytes) = storage::load() {
            if let Ok(saved) = astrobrawl_shared::deserialize_packet::<PlayerSaveData>(&bytes) {
                self.local_ship.apply_save_data(&saved);
            }
        }
    }

    pub fn save_progression(&self) {
        let save_data = self.local_ship.to_save_data();
        if let Ok(bytes) = astrobrawl_shared::serialize_packet(&save_data) {
            storage::save(&bytes);
        }
    }


    fn try_connect(&mut self, url: &str, token: &str) {
        self.connecting = true;
        self.status_text = format!("Connexion à {}...", url);

        let username = storage::get_player_name();
        let safe_username: String = username.chars().filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-').collect();
        let tok = if token.is_empty() { "guest" } else { token };
        let guest_id = storage::get_guest_id();
        let full_url = format!("{url}?token={tok}&username={safe_username}&guest_id={guest_id}");

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
        let is_connected_now = net::is_connected();

        if !self.connected && is_connected_now {
            self.connected = true;
            self.connecting = false;
            self.is_offline_sim = false;
            self.status_text = "Connecté au serveur spatial multijoueur (SYNC TEMPS RÉEL)".to_string();

            storage::set_connection_status(1);

            let token = storage::get_auth_token();
            let auth_msg = ClientMessage::Auth {
                token,
            };
            self.send_message(&auth_msg);

            // Sync current progression to server
            let save_data = self.local_ship.to_save_data();
            self.send_message(&ClientMessage::SyncProgression { save: save_data });
        } else if self.connected && !is_connected_now {
            // Disconnected from server
            self.connected = false;
            self.connecting = false;
            self.is_offline_sim = false;
            self.status_text = "Déconnecté du serveur... reconnexion automatique en cours".to_string();

            storage::set_connection_status(0);
        }

        // Automatic Reconnection loop: retry every 1.5 seconds if disconnected
        let now = macroquad::time::get_time();
        if !self.connected && (now - self.last_reconnect_attempt > 1.5) {
            self.last_reconnect_attempt = now;
            let url = storage::get_server_url();
            let token = storage::get_auth_token();
            self.try_connect(&url, &token);
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

                let server_has_progress = ship.level > 1 || ship.xp > 0 || ship.credits != 1000 || ship.cargo.used_capacity() > 0 || ship.talent_points > 0;
                let client_has_progress = self.local_ship.level > 1 || self.local_ship.xp > 0 || self.local_ship.credits != 1000 || self.local_ship.cargo.used_capacity() > 0 || self.local_ship.talent_points > 0;

                if server_has_progress || !client_has_progress {
                    self.local_ship = ship;
                } else {
                    self.local_ship.id = player_id;
                    self.local_ship.username = username.clone();
                    let save = self.local_ship.to_save_data();
                    self.send_message(&ClientMessage::SyncProgression { save });
                }

                self.current_map = current_map;
                self.camera_pos = self.local_ship.position;
                self.connected = true;
                self.is_offline_sim = false;
                storage::set_connection_status(1);
                self.status_text = format!("Pilote: {} (En Ligne)", username);
                self.show_notification(
                    "Connexion Établie",
                    &format!("Bienvenue commandant {}", username),
                    [0.0, 1.0, 0.6, 1.0],
                );
                self.save_progression();
            }
            ServerMessage::AuthError { message } => {
                self.status_text = format!("Erreur: {}", message);
                self.connected = false;
                self.is_offline_sim = false;
                storage::set_connection_status(0);
            }
            ServerMessage::WorldSnapshot(snapshot) => {
                self.connected = true;
                self.is_offline_sim = false;

                if let Some(p) = snapshot.players.iter().find(|p| p.id == self.local_player_id) {
                    if self.local_ship.is_alive && !p.is_alive {
                        audio::play(audio::SFX_EXPLOSION);
                    } else if p.is_alive {
                        if p.shield < self.local_ship.shield {
                            audio::play(audio::SFX_SHIELD_HIT);
                        } else if p.health < self.local_ship.health {
                            audio::play(audio::SFX_HULL_HIT);
                        }
                        if p.level > self.local_ship.level {
                            audio::play(audio::SFX_LEVEL_UP);
                        } else if p.credits > self.local_ship.credits || p.cargo.used_capacity() > self.local_ship.cargo.used_capacity() {
                            audio::play(audio::SFX_COLLECT);
                        }
                        if p.is_in_safe_zone && !self.local_ship.is_in_safe_zone {
                            audio::play(audio::SFX_ZONE_SAFE);
                        }
                    }
                }

                if snapshot.aliens.len() < self.aliens.len() && !self.aliens.is_empty() {
                    audio::play(audio::SFX_EXPLOSION);
                }

                self.current_map = snapshot.current_map;
                self.lasers = snapshot.lasers;
                self.minerals = snapshot.minerals;
                self.aliens = snapshot.aliens;
                self.loot_boxes = snapshot.loot_boxes;
                self.portals = snapshot.portals;

                let mut current_ids = std::collections::HashSet::new();
                for p in snapshot.players {
                    current_ids.insert(p.id);
                    if p.id == self.local_player_id {
                        let prev_credits = self.local_ship.credits;
                        let prev_xp = self.local_ship.xp;
                        let prev_level = self.local_ship.level;
                        let prev_cargo = self.local_ship.cargo.clone();

                        self.local_ship.health = p.health;
                        self.local_ship.max_health = p.max_health;
                        self.local_ship.shield = p.shield;
                        self.local_ship.max_shield = p.max_shield;
                        self.local_ship.credits = p.credits;
                        self.local_ship.score = p.score;
                        self.local_ship.level = p.level;
                        self.local_ship.xp = p.xp;
                        self.local_ship.cargo = p.cargo.clone();
                        self.local_ship.is_alive = p.is_alive;
                        self.local_ship.is_in_safe_zone = p.is_in_safe_zone;
                        if self.local_ship.position.distance_to(p.position) > 100.0 {
                            self.local_ship.position = p.position;
                            self.local_ship.velocity = p.velocity;
                        }

                        if prev_credits != p.credits || prev_xp != p.xp || prev_level != p.level || prev_cargo != p.cargo {
                            self.save_progression();
                        }
                    }
                    self.players.insert(p.id, p);
                }
                // Purge disconnected players
                self.players.retain(|id, _| current_ids.contains(id));
            }
            ServerMessage::PlayerKilled { victim_id, .. } => {
                if victim_id == self.local_player_id {
                    audio::play(audio::SFX_EXPLOSION);
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
        if in_safe_zone && !self.local_ship.is_in_safe_zone {
            audio::play(audio::SFX_ZONE_SAFE);
        }
        self.local_ship.is_in_safe_zone = in_safe_zone;

        // Space Base health and shield regeneration
        if in_safe_zone && self.local_ship.is_alive {
            let regen = 25.0 * dt;
            self.local_ship.health = (self.local_ship.health + regen).min(self.local_ship.max_health);
            self.local_ship.shield = (self.local_ship.shield + regen * 1.5).min(self.local_ship.max_shield);
        } else if self.local_ship.is_alive {
            let regen = 4.0 * dt * (1.0 + self.local_ship.talents.defense_regen as f32 * 0.15);
            self.local_ship.shield = (self.local_ship.shield + regen).min(self.local_ship.max_shield);

            // Low Hull Alert Warning
            let now_f = get_time() as f32;
            if self.local_ship.health < self.local_ship.max_health * 0.30 {
                if now_f - self.alarm_sound_timer > 1.4 {
                    self.alarm_sound_timer = now_f;
                    audio::play(audio::SFX_ALARM);
                }
            }
        }

        let now = get_time() as f32;

        // --- Map 1-1 Aliens: Streuners only, in small number (6), non-aggressive unless attacked ---
        self.aliens.retain(|a| a.alien_type == AlienType::Streuner);
        let streuner_count = self.aliens.iter().filter(|a| a.health > 0.0 && a.alien_type == AlienType::Streuner).count();

        // Replenish Streuners (Map 1-1: 6 Streuners maximum)
        if streuner_count < 6 {
            let seed = (self.next_entity_id as f32 + now * 13.37) * 31.41;
            let angle = (seed.sin() * 43758.5453).fract() * std::f32::consts::TAU;
            let dist = 550.0 + ((seed + 2.0).sin() * 43758.5453).fract().abs() * 700.0;
            let spawn_pos = Vec2::new(angle.cos() * dist, angle.sin() * dist);
            if spawn_pos.distance_to(player_pos) > 350.0 {
                self.aliens.push(Alien {
                    id: self.next_entity_id,
                    alien_type: AlienType::Streuner,
                    position: spawn_pos,
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
        }

        // Alien AI Simulation
        let mut new_alien_lasers = Vec::new();
        let mut dropped_loot = Vec::new();

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

            // Streuner is completely NON-AGGRESSIVE unless attacked by the player!
            let dist_to_player = alien.position.distance_to(player_pos);
            let is_retaliating = alien.target_player_id == Some(self.local_player_id);

            // Disengage and return to peaceful if player is in safe base or retreats beyond 700m
            if in_safe_zone || dist_to_player > 700.0 || !self.local_ship.is_alive {
                alien.target_player_id = None;
            }

            if is_retaliating && !in_safe_zone && self.local_ship.is_alive {
                // Aggro player (retaliation after being attacked)
                let dir = (player_pos - alien.position).normalize();
                alien.rotation = dir.y.atan2(dir.x);
                alien.velocity = dir * alien.alien_type.speed();
                alien.position += alien.velocity * dt;

                // Alien Shoot
                if now - alien.last_shot_time >= alien.alien_type.laser_cooldown() && dist_to_player < 420.0 {
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
                // Peaceful Idle wander
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
                // Player or other pilot laser hits aliens
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
                            if laser.shooter_id == self.local_player_id {
                                audio::play(audio::SFX_SHIELD_HIT);
                            }
                        } else {
                            alien.health -= dmg;
                            if laser.shooter_id == self.local_player_id {
                                audio::play(audio::SFX_HULL_HIT);
                            }
                        }

                        if alien.health <= 0.0 {
                            hit_alien_ids.push((alien.id, laser.shooter_id));
                        }
                        break;
                    }
                }
            } else {
                // Alien laser hits local player
                if !in_safe_zone && self.local_ship.is_alive && laser.position.distance_to(player_pos) < SHIP_RADIUS + 4.0 {
                    laser.lifetime = 0.0;
                    let dmg = laser.damage;
                    if self.local_ship.shield > 0.0 {
                        let absorbed = dmg.min(self.local_ship.shield);
                        self.local_ship.shield -= absorbed;
                        let remaining = dmg - absorbed;
                        self.local_ship.health -= remaining;
                        audio::play(audio::SFX_SHIELD_HIT);
                    } else {
                        self.local_ship.health -= dmg;
                        audio::play(audio::SFX_HULL_HIT);
                    }

                    if self.local_ship.health <= 0.0 {
                        self.local_ship.health = 0.0;
                        self.local_ship.is_alive = false;
                        player_destroyed = true;
                    }
                }

                // Alien laser can also hit other pilots
                if laser.lifetime > 0.0 {
                    for (pid, pilot) in self.players.iter_mut() {
                        if *pid != self.local_player_id && pilot.is_alive && laser.position.distance_to(pilot.position) < SHIP_RADIUS + 4.0 {
                            laser.lifetime = 0.0;
                            let dmg = laser.damage;
                            if pilot.shield > 0.0 {
                                let absorbed = dmg.min(pilot.shield);
                                pilot.shield -= absorbed;
                                pilot.health -= dmg - absorbed;
                            } else {
                                pilot.health -= dmg;
                            }
                            if pilot.health <= 0.0 {
                                pilot.health = 0.0;
                                pilot.is_alive = false;
                            }
                            break;
                        }
                    }
                }
            }
        }

        if player_destroyed {
            audio::play(audio::SFX_EXPLOSION);
            self.spawn_explosion(player_pos, RED, 40);
            self.spawn_explosion(player_pos, ORANGE, 30);

            // Drop player cargo box upon death (lasts 60s / 1 min)
            let drop_creds = (self.local_ship.credits / 10).min(500);
            let drop_scrap = self.local_ship.cargo.scrap / 2;
            dropped_loot.push(LootBox {
                id: self.next_entity_id,
                position: player_pos,
                credits: drop_creds.max(50),
                mineral: Some((MineralType::Prometium, 2)),
                scrap: drop_scrap.max(1),
                plasma_cores: 0,
                lifetime: 60.0,
                owner_id: None,
            });
            self.next_entity_id += 1;
        }

        self.lasers.retain(|l| l.lifetime > 0.0);

        // Handle Dead Aliens & Drop Loot Boxes (lasts 1 min, only killer can pick up)
        for (dead_id, killer_id) in hit_alien_ids {
            if let Some(pos) = self.aliens.iter().position(|a| a.id == dead_id) {
                let alien = self.aliens.remove(pos);
                audio::play(audio::SFX_EXPLOSION);
                self.spawn_explosion(alien.position, ORANGE, 30);
                self.spawn_explosion(alien.position, SKYBLUE, 20);

                let xp = alien.alien_type.xp_reward();
                let credits = alien.alien_type.credits_reward();

                if killer_id == self.local_player_id {
                    if self.local_ship.add_xp(xp) {
                        audio::play(audio::SFX_LEVEL_UP);
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
                    self.save_progression();
                } else if let Some(p) = self.players.get_mut(&killer_id) {
                    p.credits += credits;
                    p.add_xp(xp);
                }

                // Drop Cargo Box with 60s lifetime and exclusive ownership for killer
                dropped_loot.push(LootBox {
                    id: self.next_entity_id,
                    position: alien.position,
                    credits: credits / 2,
                    mineral: Some((MineralType::Prometium, 3)),
                    scrap: 2,
                    plasma_cores: if alien.alien_type == AlienType::Sibelon { 2 } else { 0 },
                    lifetime: 60.0,
                    owner_id: Some(killer_id),
                });
                self.next_entity_id += 1;
            }
        }

        self.loot_boxes.extend(dropped_loot);

        // Loot Box Collection (Only allowed if player is the owner or box is unassigned)
        let mut collected_box_ids = Vec::new();
        let mut collected_notifications = Vec::new();

        for loot in &mut self.loot_boxes {
            loot.lifetime -= dt;

            let can_collect = match loot.owner_id {
                Some(owner) => owner == self.local_player_id,
                None => true,
            };

            if loot.position.distance_to(player_pos) < LOOTBOX_RADIUS + SHIP_RADIUS && self.local_ship.is_alive {
                if can_collect {
                    collected_box_ids.push(loot.id);
                    audio::play(audio::SFX_COLLECT);
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
        }

        for (cred, scr) in collected_notifications {
            self.spawn_float_text(
                format!("📦 Fret Récupéré ! (+{} C., +{} Ferraille)", cred, scr),
                player_pos + Vec2::new(0.0, -32.0),
                YELLOW,
            );
            self.save_progression();
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

                if spark_needed && get_time() as f32 - self.mining_sound_timer > 0.12 {
                    self.mining_sound_timer = get_time() as f32;
                    audio::play(audio::SFX_MINING);
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
                        audio::play(audio::SFX_COLLECT);
                        let xp_gain = m_type.value() * 2;
                        if self.local_ship.add_xp(xp_gain) {
                            audio::play(audio::SFX_LEVEL_UP);
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
                        self.save_progression();
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

        // Autosave progression periodically
        self.autosave_timer += dt;
        if self.autosave_timer >= 3.0 {
            self.autosave_timer = 0.0;
            self.save_progression();
        }
    }

    fn shoot_laser(&mut self) {
        if !self.local_ship.is_alive || self.local_ship.is_in_safe_zone {
            return;
        }

        audio::play(audio::SFX_LASER);

        // Auto-aim towards locked target (alien or other pilot) if alive and valid, else ship rotation
        let dir = if let Some(target_id) = self.locked_target_id {
            if let Some(target) = self.aliens.iter().find(|a| a.id == target_id && a.health > 0.0) {
                (target.position - self.local_ship.position).normalize()
            } else if let Some(target_p) = self.players.get(&target_id) {
                if target_p.is_alive {
                    (target_p.position - self.local_ship.position).normalize()
                } else {
                    Vec2::new(self.local_ship.rotation.cos(), self.local_ship.rotation.sin())
                }
            } else {
                Vec2::new(self.local_ship.rotation.cos(), self.local_ship.rotation.sin())
            }
        } else {
            Vec2::new(self.local_ship.rotation.cos(), self.local_ship.rotation.sin())
        };

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
    let font_bytes = include_bytes!("../assets/Rajdhani-SemiBold.ttf");
    let custom_font = load_ttf_font_from_bytes(font_bytes).ok();

    let mut game = GameClient::new();

    // Connect to server using dynamic configured URL from HTML/localStorage
    let server_url = storage::get_server_url();
    let token = storage::get_auth_token();
    game.try_connect(&server_url, &token);

    let mut last_shot_time = 0.0f32;

    loop {
        let dt = get_frame_time().min(0.05);

        // Network polling
        game.poll_network();

        // Sync player username from localStorage (from HTML modal callsign)
        let stored_name = storage::get_player_name();
        if !stored_name.is_empty() && (game.local_ship.username == "Pilote Spatial" || game.local_ship.username.is_empty()) {
            game.local_ship.username = stored_name;
        }

        let now = get_time() as f32;
        if (now as f64) - game.last_ping_send > 2.0 && game.connected {
            game.last_ping_send = now as f64;
            let ping = ClientMessage::Ping {
                client_time: (now * 1000.0) as u64,
            };
            game.send_message(&ping);
        }

        if !game.connected {
            // Completely freeze inputs and gameplay simulation while disconnected
            clear_background(Color::new(0.015, 0.02, 0.05, 1.0));
            for star in &game.stars {
                let sx = (star.pos.x - game.camera_pos.x * star.layer).rem_euclid(screen_width() + 100.0) - 50.0;
                let sy = (star.pos.y - game.camera_pos.y * star.layer).rem_euclid(screen_height() + 100.0) - 50.0;
                draw_circle(sx, sy, star.size, star.color);
            }
            next_frame().await;
            continue;
        }

        // --- Controls & Inputs ---
        let mut move_vec = Vec2::ZERO;
        let mut target_angle = game.local_ship.rotation;
        let mut shoot = false;
        let mut respawn = false;

        // Modal toggle shortcuts
        if is_key_pressed(KeyCode::T) {
            game.active_modal = if game.active_modal == ActiveModal::Talents { ActiveModal::None } else { ActiveModal::Talents };
            audio::play(audio::SFX_UI_CLICK);
        }
        if is_key_pressed(KeyCode::H) || is_key_pressed(KeyCode::B) {
            game.active_modal = if game.active_modal == ActiveModal::Hangar { ActiveModal::None } else { ActiveModal::Hangar };
            audio::play(audio::SFX_UI_CLICK);
        }
        if is_key_pressed(KeyCode::C) {
            game.active_modal = if game.active_modal == ActiveModal::Crafting { ActiveModal::None } else { ActiveModal::Crafting };
            audio::play(audio::SFX_UI_CLICK);
        }

        // Audio Mute toggle shortcut (M)
        if is_key_pressed(KeyCode::M) {
            game.is_muted = audio::toggle_mute();
            let status = if game.is_muted { "🔇 Audio : Désactivé (Muet)" } else { "🔊 Audio : Activé (Sons sci-fi)" };
            game.spawn_float_text(
                status.to_string(),
                game.local_ship.position + Vec2::new(0.0, -35.0),
                if game.is_muted { mocha::RED } else { mocha::GREEN },
            );
        }

        // Tab Targeting: Select or cycle nearest alien or other pilot
        if is_key_pressed(KeyCode::Tab) {
            let p_pos = game.local_ship.position;
            let mut candidates: Vec<(u64, f32, String)> = Vec::new();

            for a in &game.aliens {
                if a.health > 0.0 {
                    let d = p_pos.distance_to(a.position);
                    if d < 1350.0 {
                        candidates.push((a.id, d, format!("👽 {}", a.alien_type.name())));
                    }
                }
            }

            for p in game.players.values() {
                if p.id != game.local_player_id && p.is_alive {
                    let d = p_pos.distance_to(p.position);
                    if d < 1350.0 {
                        candidates.push((p.id, d, format!("👨‍🚀 {}", p.username)));
                    }
                }
            }

            candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

            if !candidates.is_empty() {
                if let Some(cur_id) = game.locked_target_id {
                    if let Some(pos) = candidates.iter().position(|(id, _, _)| *id == cur_id) {
                        let next_idx = (pos + 1) % candidates.len();
                        game.locked_target_id = Some(candidates[next_idx].0);
                    } else {
                        game.locked_target_id = Some(candidates[0].0);
                    }
                } else {
                    game.locked_target_id = Some(candidates[0].0);
                }

                audio::play(audio::SFX_TARGET_LOCK);

                if let Some(locked_id) = game.locked_target_id {
                    if let Some((_, _, label)) = candidates.iter().find(|(id, _, _)| *id == locked_id) {
                        game.spawn_float_text(
                            format!("🎯 CIBLE: {}", label),
                            game.local_ship.position + Vec2::new(0.0, -42.0),
                            GOLD,
                        );
                    }
                }
            } else {
                if game.locked_target_id.is_some() {
                    audio::play(audio::SFX_TARGET_LOST);
                }
                game.locked_target_id = None;
            }
        }

        if is_key_pressed(KeyCode::Escape) {
            if game.active_modal != ActiveModal::None {
                game.active_modal = ActiveModal::None;
                audio::play(audio::SFX_UI_CLICK);
            } else if game.locked_target_id.is_some() {
                game.locked_target_id = None;
                audio::play(audio::SFX_TARGET_LOST);
            }
        }

        // Sell Cargo shortcut at space base
        if is_key_pressed(KeyCode::V) && game.local_ship.is_in_safe_zone {
            let credits = game.local_ship.cargo.sell_all_minerals();
            if credits > 0 {
                audio::play(audio::SFX_COLLECT);
                game.local_ship.credits += credits;
                let pos = game.local_ship.position;
                game.spawn_float_text(format!("💰 Minerais Vendus : +{} C.", credits), pos + Vec2::new(0.0, -35.0), GOLD);
                game.save_progression();
                game.send_message(&ClientMessage::SellCargo);
            }
        }

        if game.local_ship.is_alive && game.active_modal == ActiveModal::None {
            // 8-Directional ZQSD Movement (NO SHIFT REQUIRED!)
            // Z / W / Up = Forward/Up
            if is_key_down(KeyCode::Z) || is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
                move_vec.y -= 1.0;
            }
            // S / Down = Backward/Down
            if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
                move_vec.y += 1.0;
            }
            // Q / A / Left = Left
            if is_key_down(KeyCode::Q) || is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
                move_vec.x -= 1.0;
            }
            // D / Right = Right
            if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
                move_vec.x += 1.0;
            }

            // Right click = fly towards mouse directly (no shift key!)
            if is_mouse_button_down(MouseButton::Right) {
                let (mx, my) = mouse_position();
                let screen_center = Vec2::new(screen_width() * 0.5, screen_height() * 0.5);
                let mouse_dir = Vec2::new(mx - screen_center.x, my - screen_center.y);
                if mouse_dir.length_squared() > 100.0 {
                    move_vec += mouse_dir.normalize();
                }
            }

            // Click directly on an alien to lock target
            if is_mouse_button_pressed(MouseButton::Left) {
                let (mx, my) = mouse_position();
                let click_world = Vec2::new(
                    mx - screen_width() * 0.5 + game.camera_pos.x,
                    my - screen_height() * 0.5 + game.camera_pos.y,
                );
                for alien in &game.aliens {
                    if alien.health > 0.0 && alien.position.distance_to(click_world) < 42.0 {
                        game.locked_target_id = Some(alien.id);
                        audio::play(audio::SFX_TARGET_LOCK);
                        game.spawn_float_text(
                            format!("🎯 CIBLE: {}", alien.alien_type.name()),
                            alien.position + Vec2::new(0.0, -35.0),
                            GOLD,
                        );
                        break;
                    }
                }
            }

            // Auto-Aiming: Face locked target if present, else face mouse
            let mut auto_aimed = false;
            if let Some(target_id) = game.locked_target_id {
                if let Some(alien) = game.aliens.iter().find(|a| a.id == target_id && a.health > 0.0) {
                    let dist = game.local_ship.position.distance_to(alien.position);
                    if dist < 1400.0 {
                        let aim_dir = (alien.position - game.local_ship.position).normalize();
                        target_angle = aim_dir.y.atan2(aim_dir.x);
                        auto_aimed = true;
                    } else {
                        game.locked_target_id = None;
                        audio::play(audio::SFX_TARGET_LOST);
                    }
                } else {
                    game.locked_target_id = None;
                }
            }

            if !auto_aimed {
                let (mx, my) = mouse_position();
                let screen_center = Vec2::new(screen_width() * 0.5, screen_height() * 0.5);
                let mouse_dir = Vec2::new(mx - screen_center.x, my - screen_center.y);
                target_angle = mouse_dir.y.atan2(mouse_dir.x);
            }

            // Laser Fire (Left Click or Space)
            let talent_rate_bonus = 1.0 + (game.local_ship.talents.combat_fire_rate as f32 * 0.05);
            let cooldown = game.local_ship.ship_class.laser_cooldown() / talent_rate_bonus;
            if (is_mouse_button_down(MouseButton::Left) || is_key_down(KeyCode::Space)) && now - last_shot_time >= cooldown {
                shoot = true;
                last_shot_time = now;
            }

            // Mining Laser Action (Press or hold E - NO SHIFT REQUIRED!)
            if is_key_down(KeyCode::E) {
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
                    audio::play(audio::SFX_WARP);
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
            let is_moving = move_vec.length_squared() > 0.001;
            if is_moving {
                let thrust_dir = move_vec.normalize();
                let back_dir = Vec2::new(-thrust_dir.x, -thrust_dir.y);
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

        // Apply physics locally with 8-directional move_vec
        apply_ship_physics(&mut game.local_ship, move_vec, target_angle, dt);

        if respawn {
            audio::play(audio::SFX_RESPAWN);
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
                thrust: move_vec.length_squared() > 0.001,
                target_angle,
                move_vec,
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
        let bn_dim = measure_crisp_text(custom_font.as_ref(), base_name, 16.0);
        draw_crisp_text_shadow(custom_font.as_ref(), base_name, base_sx - bn_dim.width * 0.5, base_sy - 85.0, 16.0, mocha::SKY);

        if game.local_ship.is_in_safe_zone {
            let z_text = "🛡️ ZONE DE NON-AGRESSION • SÉCURITÉ ACTIVE [V: Vente | H: Hangar | C: Craft]";
            let zd = measure_crisp_text(custom_font.as_ref(), z_text, 14.0);
            draw_crisp_text_shadow(custom_font.as_ref(), z_text, base_sx - zd.width * 0.5, base_sy + 95.0, 14.0, mocha::GREEN);
        }

        // 4. Portals & Event Rifts
        for portal in &game.portals {
            let px = portal.position.x - game.camera_pos.x + half_screen.x;
            let py = portal.position.y - game.camera_pos.y + half_screen.y;

            let (p_col, p_label) = match &portal.portal_type {
                PortalType::MapJump { target_map, .. } => (mocha::SAPPHIRE, target_map.name()),
                PortalType::EventRift { event_type, .. } => {
                    let rgba = event_type.color_rgba();
                    (Color::new(rgba[0], rgba[1], rgba[2], 0.9), event_type.name())
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

            let pld = measure_crisp_text(custom_font.as_ref(), p_label, 14.0);
            draw_crisp_text_shadow(custom_font.as_ref(), p_label, px - pld.width * 0.5, py - portal.radius - 12.0, 14.0, p_col);

            if game.local_ship.position.distance_to(portal.position) <= portal.radius + 30.0 {
                let jump_hint = "[J] ENTRER DANS LE PORTAIL";
                let jhd = measure_crisp_text(custom_font.as_ref(), jump_hint, 15.0);
                draw_crisp_text_shadow(custom_font.as_ref(), jump_hint, px - jhd.width * 0.5, py + portal.radius + 20.0, 15.0, mocha::YELLOW);
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

        // 8. Loot Boxes (Boîtes de Fret avec minuteur 1 min et indicateur propriétaire)
        for loot in &game.loot_boxes {
            let lx = loot.position.x - game.camera_pos.x + half_screen.x;
            let ly = loot.position.y - game.camera_pos.y + half_screen.y;

            let size = 16.0;
            let is_mine = match loot.owner_id {
                Some(owner) => owner == game.local_player_id,
                None => true,
            };

            let border_col = if is_mine { mocha::YELLOW } else { mocha::RED };
            let bg_col = if is_mine { Color::new(0.1, 0.6, 0.9, 0.85) } else { Color::new(0.4, 0.1, 0.15, 0.85) };

            draw_rectangle(lx - size * 0.5, ly - size * 0.5, size, size, bg_col);
            draw_rectangle_lines(lx - size * 0.5, ly - size * 0.5, size, size, 2.0, border_col);
            draw_circle(lx, ly, 3.0, WHITE);

            let remaining_s = (loot.lifetime.max(0.0).ceil()) as u32;
            let box_text = if is_mine {
                format!("📦 FRET ({}s)", remaining_s)
            } else {
                format!("🔒 FRET RÉSERVÉ ({}s)", remaining_s)
            };
            let bwd = measure_crisp_text(custom_font.as_ref(), &box_text, 12.0);
            draw_crisp_text_shadow(custom_font.as_ref(), &box_text, lx - bwd.width * 0.5, ly - 14.0, 12.0, border_col);
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
            let is_hostile = alien.target_player_id == Some(game.local_player_id);
            let (nose_len, side_len, alien_col) = match alien.alien_type {
                AlienType::Streuner => (20.0, 16.0, if is_hostile { mocha::RED } else { mocha::YELLOW }),
                AlienType::Lordakia => (26.0, 22.0, mocha::PEACH),
                AlienType::Sibelon => (44.0, 40.0, mocha::MAUVE),
            };

            let nose = to_mq(Vec2::new(ax + rot.cos() * nose_len, ay + rot.sin() * nose_len));
            let left = to_mq(Vec2::new(ax + (rot + 2.4).cos() * side_len, ay + (rot + 2.4).sin() * side_len));
            let right = to_mq(Vec2::new(ax + (rot - 2.4).cos() * side_len, ay + (rot - 2.4).sin() * side_len));

            draw_triangle(nose, left, right, if is_hostile { Color::new(0.2, 0.05, 0.08, 1.0) } else { Color::new(0.18, 0.15, 0.08, 1.0) });
            draw_triangle_lines(nose, left, right, 2.0, alien_col);

            // Alien HP Bar
            let bar_w = side_len * 2.0;
            let bar_h = 3.5;
            let hp_pct = (alien.health / alien.max_health).clamp(0.0, 1.0);
            draw_rectangle(ax - bar_w * 0.5, ay - side_len - 14.0, bar_w, bar_h, mocha::SURFACE0);
            draw_rectangle(ax - bar_w * 0.5, ay - side_len - 14.0, bar_w * hp_pct, bar_h, if is_hostile { mocha::RED } else { mocha::YELLOW });

            let a_name = if alien.alien_type == AlienType::Streuner {
                if is_hostile {
                    "Streuner [⚠️ HOSTILE]"
                } else {
                    "Streuner [Passif]"
                }
            } else {
                alien.alien_type.name()
            };
            let and = measure_crisp_text(custom_font.as_ref(), a_name, 13.0);
            draw_crisp_text_shadow(custom_font.as_ref(), a_name, ax - and.width * 0.5, ay - side_len - 18.0, 13.0, if is_hostile { mocha::RED } else { mocha::SUBTEXT1 });

            // Draw Dark Orbit Target Lock Reticle
            if game.locked_target_id == Some(alien.id) {
                let box_size = side_len * 2.5 + ((now * 8.0).sin() * 2.0);
                let half_b = box_size * 0.5;
                let bracket_len = 10.0;
                let target_col = mocha::RED;

                // 4 Corner Brackets
                draw_line(ax - half_b, ay - half_b, ax - half_b + bracket_len, ay - half_b, 2.5, target_col);
                draw_line(ax - half_b, ay - half_b, ax - half_b, ay - half_b + bracket_len, 2.5, target_col);

                draw_line(ax + half_b, ay - half_b, ax + half_b - bracket_len, ay - half_b, 2.5, target_col);
                draw_line(ax + half_b, ay - half_b, ax + half_b, ay - half_b + bracket_len, 2.5, target_col);

                draw_line(ax - half_b, ay + half_b, ax - half_b + bracket_len, ay + half_b, 2.5, target_col);
                draw_line(ax - half_b, ay + half_b, ax - half_b, ay - half_b + bracket_len, 2.5, target_col);

                draw_line(ax + half_b, ay + half_b, ax + half_b - bracket_len, ay + half_b, 2.5, target_col);
                draw_line(ax + half_b, ay + half_b, ax + half_b, ay + half_b - bracket_len, 2.5, target_col);

                let dist_m = (game.local_ship.position.distance_to(alien.position)) as u32;
                let lock_str = format!("🎯 CIBLE: {}m", dist_m);
                let ldim = measure_crisp_text(custom_font.as_ref(), &lock_str, 13.0);
                draw_crisp_text_shadow(custom_font.as_ref(), &lock_str, ax - ldim.width * 0.5, ay + half_b + 18.0, 13.0, mocha::YELLOW);
            }
        }

        // 11. Other Multiplayer & Simulated Pilots
        for player in game.players.values() {
            if player.id == game.local_player_id || !player.is_alive {
                continue;
            }

            let px = player.position.x - game.camera_pos.x + half_screen.x;
            let py = player.position.y - game.camera_pos.y + half_screen.y;
            let rot = player.rotation;

            let nose = to_mq(Vec2::new(px + rot.cos() * 24.0, py + rot.sin() * 24.0));
            let left_wing = to_mq(Vec2::new(px + (rot + 2.45).cos() * 21.0, py + (rot + 2.45).sin() * 21.0));
            let right_wing = to_mq(Vec2::new(px + (rot - 2.45).cos() * 21.0, py + (rot - 2.45).sin() * 21.0));
            let engine = to_mq(Vec2::new(px + (rot + std::f32::consts::PI).cos() * 14.0, py + (rot + std::f32::consts::PI).sin() * 14.0));

            // Thruster flame when moving
            if player.is_thrusting {
                let flame_len = 14.0 + ((now * 28.0).sin() * 3.5);
                let flame_tip = to_mq(Vec2::new(px + (rot + std::f32::consts::PI).cos() * (14.0 + flame_len), py + (rot + std::f32::consts::PI).sin() * (14.0 + flame_len)));
                draw_triangle(left_wing * 0.4 + engine * 0.6, flame_tip, right_wing * 0.4 + engine * 0.6, mocha::PEACH);
            }

            // Ship class specific hull & border colors
            let (hull_col, border_col) = match player.ship_class {
                ShipClass::Combat => (Color::new(0.14, 0.24, 0.45, 1.0), mocha::BLUE),
                ShipClass::Minier => (Color::new(0.48, 0.38, 0.12, 1.0), mocha::YELLOW),
                ShipClass::Transport => (Color::new(0.42, 0.25, 0.16, 1.0), mocha::PEACH),
                ShipClass::Exploration => (Color::new(0.16, 0.40, 0.28, 1.0), mocha::GREEN),
            };

            draw_triangle(nose, left_wing, engine, hull_col);
            draw_triangle(nose, right_wing, engine, hull_col);
            draw_triangle_lines(nose, left_wing, engine, 1.8, border_col);
            draw_triangle_lines(nose, right_wing, engine, 1.8, border_col);

            // Cockpit glass
            let cockpit = Vec2::new(px + rot.cos() * 6.0, py + rot.sin() * 6.0);
            draw_circle(cockpit.x, cockpit.y, 3.8, mocha::SAPPHIRE);

            // Mining laser beam if mining
            if player.is_mining {
                if let Some(target_m_id) = player.mining_target {
                    if let Some(mineral) = game.minerals.iter().find(|m| m.id == target_m_id) {
                        let mx = mineral.position.x - game.camera_pos.x + half_screen.x;
                        let my = mineral.position.y - game.camera_pos.y + half_screen.y;
                        let beam_w = 2.0 + ((now * 25.0).sin() * 1.0).abs();
                        draw_line(cockpit.x, cockpit.y, mx, my, beam_w, mocha::YELLOW);
                        draw_circle(mx, my, 4.0, mocha::YELLOW);
                    }
                }
            }

            // Health & Shield Bars Above Other Ship
            let p_bar_w = 50.0;
            let p_bar_x = px - p_bar_w * 0.5;
            let p_sh_p = (player.shield / player.max_shield).clamp(0.0, 1.0);
            draw_rectangle(p_bar_x, py - 34.0, p_bar_w, 4.0, mocha::SURFACE0);
            draw_rectangle(p_bar_x, py - 34.0, p_bar_w * p_sh_p, 4.0, mocha::SAPPHIRE);
            let p_hp_p = (player.health / player.max_health).clamp(0.0, 1.0);
            draw_rectangle(p_bar_x, py - 28.0, p_bar_w, 4.0, mocha::SURFACE0);
            draw_rectangle(p_bar_x, py - 28.0, p_bar_w * p_hp_p, 4.0, mocha::RED);

            // Target Lock Brackets around other player if targeted
            if game.locked_target_id == Some(player.id) {
                let bracket_len = 9.0;
                let half_b = 32.0;
                let target_col = mocha::YELLOW;
                draw_line(px - half_b, py - half_b, px - half_b + bracket_len, py - half_b, 2.5, target_col);
                draw_line(px - half_b, py - half_b, px - half_b, py - half_b + bracket_len, 2.5, target_col);
                draw_line(px + half_b, py - half_b, px + half_b - bracket_len, py - half_b, 2.5, target_col);
                draw_line(px + half_b, py - half_b, px + half_b - bracket_len, py - half_b, 2.5, target_col);
                draw_line(px - half_b, py + half_b, px - half_b + bracket_len, py + half_b, 2.5, target_col);
                draw_line(px - half_b, py + half_b, px - half_b, py + half_b + bracket_len, 2.5, target_col);
                draw_line(px + half_b, py + half_b, px + half_b - bracket_len, py + half_b, 2.5, target_col);
                draw_line(px + half_b, py + half_b, px + half_b - bracket_len, py + half_b, 2.5, target_col);

                let dist_m = (game.local_ship.position.distance_to(player.position)) as u32;
                let lock_str = format!("🎯 PILOTE: {}m", dist_m);
                let ldim = measure_crisp_text(custom_font.as_ref(), &lock_str, 13.0);
                draw_crisp_text_shadow(custom_font.as_ref(), &lock_str, px - ldim.width * 0.5, py + half_b + 20.0, 13.0, mocha::YELLOW);
            }

            // Pseudo & Class Badge Below Other Ship
            let uname = &player.username;
            let class_str = format!("Lv.{} • {}", player.level, player.ship_class.name());
            let u_dim = measure_crisp_text(custom_font.as_ref(), uname, 14.0);
            let c_dim = measure_crisp_text(custom_font.as_ref(), &class_str, 11.0);
            let pb_w = (u_dim.width.max(c_dim.width) + 22.0).max(74.0);
            let pb_h = 32.0;
            let pb_x = px - pb_w * 0.5;
            let pb_y = py + 36.0;

            draw_rectangle(pb_x, pb_y, pb_w, pb_h, Color::new(0.09, 0.09, 0.15, 0.90));
            draw_rectangle_lines(pb_x, pb_y, pb_w, pb_h, 1.2, border_col);
            draw_crisp_text_shadow(custom_font.as_ref(), uname, px - u_dim.width * 0.5, pb_y + 15.0, 14.0, mocha::PEACH);
            draw_crisp_text(custom_font.as_ref(), &class_str, px - c_dim.width * 0.5, pb_y + 27.0, 11.0, mocha::SKY);
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
                draw_triangle(left_wing * 0.4 + engine * 0.6, flame_tip, right_wing * 0.4 + engine * 0.6, mocha::PEACH);
            }

            let (hull_col, border_col) = match game.local_ship.ship_class {
                ShipClass::Combat => (Color::new(0.12, 0.28, 0.52, 1.0), mocha::BLUE),
                ShipClass::Minier => (Color::new(0.55, 0.42, 0.15, 1.0), mocha::YELLOW),
                ShipClass::Transport => (Color::new(0.48, 0.28, 0.18, 1.0), mocha::PEACH),
                ShipClass::Exploration => (Color::new(0.18, 0.45, 0.32, 1.0), mocha::GREEN),
            };

            draw_triangle(nose, left_wing, engine, hull_col);
            draw_triangle(nose, right_wing, engine, hull_col);
            draw_triangle_lines(nose, left_wing, engine, 2.0, border_col);
            draw_triangle_lines(nose, right_wing, engine, 2.0, border_col);

            // Cockpit glass
            let cockpit = Vec2::new(sx + rot.cos() * 7.0, sy + rot.sin() * 7.0);
            draw_circle(cockpit.x, cockpit.y, 4.2, mocha::SAPPHIRE);

            // Mining pods if Minier
            if game.local_ship.ship_class == ShipClass::Minier {
                let pod1 = Vec2::new(sx + (rot + 1.6).cos() * 16.0, sy + (rot + 1.6).sin() * 16.0);
                let pod2 = Vec2::new(sx + (rot - 1.6).cos() * 16.0, sy + (rot - 1.6).sin() * 16.0);
                draw_circle(pod1.x, pod1.y, 4.0, mocha::YELLOW);
                draw_circle(pod2.x, pod2.y, 4.0, mocha::YELLOW);
            }

            // Shield & HP Bars ABOVE Ship
            let bar_w = 54.0;
            let bar_h = 4.5;
            let bar_x = sx - bar_w * 0.5;

            let shield_pct = (game.local_ship.shield / game.local_ship.max_shield).clamp(0.0, 1.0);
            draw_rectangle(bar_x, sy - 34.0, bar_w, bar_h, mocha::SURFACE0);
            draw_rectangle(bar_x, sy - 34.0, bar_w * shield_pct, bar_h, mocha::SAPPHIRE);

            let hp_pct = (game.local_ship.health / game.local_ship.max_health).clamp(0.0, 1.0);
            draw_rectangle(bar_x, sy - 27.0, bar_w, bar_h, mocha::SURFACE0);
            draw_rectangle(bar_x, sy - 27.0, bar_w * hp_pct, bar_h, mocha::GREEN);

            // PSEUDO DU JOUEUR EN DESSOUS DU VAISSEAU (Demandé par l'utilisateur)
            let raw_username = if game.local_ship.username.is_empty() {
                "Commandant".to_string()
            } else {
                game.local_ship.username.clone()
            };
            let class_tag = format!("Lv.{} • {}", game.local_ship.level, game.local_ship.ship_class.name());

            let u_dim = measure_crisp_text(custom_font.as_ref(), &raw_username, 16.0);
            let t_dim = measure_crisp_text(custom_font.as_ref(), &class_tag, 12.0);
            let badge_w = (u_dim.width.max(t_dim.width) + 24.0).max(80.0);
            let badge_h = 34.0;
            let badge_x = sx - badge_w * 0.5;
            let badge_y = sy + 36.0;

            // Catppuccin Mocha Mantle Badge
            draw_rectangle(badge_x, badge_y, badge_w, badge_h, Color::new(0.09, 0.09, 0.15, 0.92));
            draw_rectangle_lines(badge_x, badge_y, badge_w, badge_h, 1.5, mocha::MAUVE);

            draw_crisp_text_shadow(custom_font.as_ref(), &raw_username, sx - u_dim.width * 0.5, badge_y + 16.0, 16.0, mocha::YELLOW);
            draw_crisp_text(custom_font.as_ref(), &class_tag, sx - t_dim.width * 0.5, badge_y + 29.0, 12.0, mocha::SKY);
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
            draw_crisp_text_shadow(custom_font.as_ref(), &ft.text, sx, sy, 16.0, col);
        }

        // 14. Top Sci-Fi HUD (Catppuccin Mocha)
        let (mouse_x, mouse_y) = mouse_position();
        let mouse_clicked = is_mouse_button_pressed(MouseButton::Left);

        let hud_w = 980.0_f32.min(screen_width() - 32.0);
        let hud_h = 44.0;
        let hud_x = (screen_width() - hud_w) * 0.5;
        let hud_y = 12.0;

        draw_rectangle(hud_x, hud_y, hud_w, hud_h, Color::new(0.09, 0.09, 0.15, 0.94));
        draw_rectangle_lines(hud_x, hud_y, hud_w, hud_h, 1.5, mocha::SURFACE1);

        // Pilot Callsign Badge (Top Bar - Prominently Displayed!)
        let pilot_name = if game.local_ship.username.is_empty() { "Commandant" } else { &game.local_ship.username };
        let pilot_pill = format!("👨‍🚀 {}", pilot_name);
        let p_dim = measure_crisp_text(custom_font.as_ref(), &pilot_pill, 15.0);
        let p_pill_w = p_dim.width + 20.0;
        draw_rectangle(hud_x + 10.0, hud_y + 7.0, p_pill_w, 30.0, Color::new(0.15, 0.15, 0.25, 0.85));
        draw_rectangle_lines(hud_x + 10.0, hud_y + 7.0, p_pill_w, 30.0, 1.2, mocha::MAUVE);
        draw_crisp_text_shadow(custom_font.as_ref(), &pilot_pill, hud_x + 20.0, hud_y + 27.0, 15.0, mocha::YELLOW);

        let mut cur_x = hud_x + 10.0 + p_pill_w + 14.0;

        let credits_str = format!("💰 {} C", game.local_ship.credits);
        draw_crisp_text_shadow(custom_font.as_ref(), &credits_str, cur_x, hud_y + 27.0, 15.0, mocha::PEACH);
        cur_x += 110.0;

        let cargo_used = game.local_ship.cargo.used_capacity();
        let cargo_max = game.local_ship.cargo.max_capacity;
        let cargo_col = if game.local_ship.cargo.is_full() { mocha::RED } else { mocha::SAPPHIRE };
        let cargo_str = format!("📦 {}/{} kg", cargo_used, cargo_max);
        draw_crisp_text_shadow(custom_font.as_ref(), &cargo_str, cur_x, hud_y + 27.0, 15.0, cargo_col);
        cur_x += 120.0;

        let level_str = format!("⭐ Lv.{} ({} / {} XP)", game.local_ship.level, game.local_ship.xp, game.local_ship.next_level_xp);
        draw_crisp_text_shadow(custom_font.as_ref(), &level_str, cur_x, hud_y + 27.0, 15.0, mocha::GREEN);
        cur_x += 170.0;

        let map_str = format!("🌐 {}", game.current_map.name());
        draw_crisp_text_shadow(custom_font.as_ref(), &map_str, cur_x, hud_y + 27.0, 14.0, mocha::LAVENDER);
        cur_x += 140.0;

        let conn_str = if game.connected {
            let other_count = game.players.len().saturating_sub(1);
            if other_count > 0 {
                format!("🟢 SERVEUR SYNC ({} pilotes)", other_count + 1)
            } else {
                "🟢 SERVEUR SYNC".to_string()
            }
        } else {
            "🔴 HORS LIGNE (Reconnexion...)".to_string()
        };
        let conn_col = if game.connected { mocha::GREEN } else { mocha::PEACH };
        draw_crisp_text_shadow(custom_font.as_ref(), &conn_str, cur_x, hud_y + 27.0, 13.0, conn_col);

        // Audio Mute Pill Button in Top HUD
        let mute_btn_w = 98.0;
        let mute_btn_x = hud_x + hud_w - mute_btn_w - 12.0;
        let mute_btn_y = hud_y + 8.0;
        let mute_hover = mouse_x >= mute_btn_x && mouse_x <= mute_btn_x + mute_btn_w && mouse_y >= mute_btn_y && mouse_y <= mute_btn_y + 28.0;

        if mute_hover && mouse_clicked {
            game.is_muted = audio::toggle_mute();
            audio::play(audio::SFX_UI_CLICK);
        }

        let (m_bg, m_border, m_txt, m_label) = if game.is_muted {
            (mocha::SURFACE0, mocha::RED, mocha::RED, "🔇 MUET [M]")
        } else {
            (if mute_hover { mocha::SURFACE1 } else { mocha::SURFACE0 }, if mute_hover { mocha::GREEN } else { mocha::SURFACE2 }, mocha::GREEN, "🔊 SON [M]")
        };
        draw_rectangle(mute_btn_x, mute_btn_y, mute_btn_w, 28.0, m_bg);
        draw_rectangle_lines(mute_btn_x, mute_btn_y, mute_btn_w, 28.0, 1.2, m_border);
        draw_crisp_text(custom_font.as_ref(), m_label, mute_btn_x + 12.0, mute_btn_y + 19.0, 13.0, m_txt);

        // 15. Notification Banner
        if game.notification_timer > 0.0 {
            let nd = measure_crisp_text(custom_font.as_ref(), &game.notification_text, 15.0);
            let nw = nd.width;
            let nx = (screen_width() - nw - 48.0) * 0.5;
            draw_rectangle(nx, 64.0, nw + 48.0, 32.0, Color::new(0.09, 0.09, 0.15, 0.95));
            draw_rectangle_lines(nx, 64.0, nw + 48.0, 32.0, 1.5, mocha::SAPPHIRE);
            draw_crisp_text_shadow(custom_font.as_ref(), &game.notification_text, nx + 24.0, 85.0, 15.0, mocha::TEXT);
        }

        // Target Info Box in HUD (Catppuccin Mocha)
        if let Some(target_id) = game.locked_target_id {
            let tw = 290.0;
            let th = 68.0;
            let tx = screen_width() - tw - 20.0;
            let ty = 66.0;

            if let Some(alien) = game.aliens.iter().find(|a| a.id == target_id && a.health > 0.0) {
                draw_rectangle(tx, ty, tw, th, Color::new(0.09, 0.09, 0.15, 0.94));
                draw_rectangle_lines(tx, ty, tw, th, 1.5, mocha::RED);

                let dist_m = game.local_ship.position.distance_to(alien.position) as u32;
                let t_title = format!("🎯 {} ({}m)", alien.alien_type.name(), dist_m);
                draw_crisp_text_shadow(custom_font.as_ref(), &t_title, tx + 12.0, ty + 20.0, 15.0, mocha::RED);

                let hp_p = (alien.health / alien.max_health).clamp(0.0, 1.0);
                draw_rectangle(tx + 12.0, ty + 28.0, 266.0, 9.0, mocha::SURFACE0);
                draw_rectangle(tx + 12.0, ty + 28.0, 266.0 * hp_p, 9.0, mocha::RED);

                let sh_p = (alien.shield / alien.max_shield).clamp(0.0, 1.0);
                draw_rectangle(tx + 12.0, ty + 41.0, 266.0, 7.0, mocha::SURFACE0);
                draw_rectangle(tx + 12.0, ty + 41.0, 266.0 * sh_p, 7.0, mocha::SAPPHIRE);

                let hp_txt = format!("{:.0} / {:.0} PV", alien.health, alien.max_health);
                draw_crisp_text(custom_font.as_ref(), &hp_txt, tx + 12.0, ty + 60.0, 11.0, mocha::SUBTEXT0);
            } else if let Some(other_player) = game.players.get(&target_id) {
                if other_player.is_alive {
                    draw_rectangle(tx, ty, tw, th, Color::new(0.09, 0.09, 0.15, 0.94));
                    draw_rectangle_lines(tx, ty, tw, th, 1.5, mocha::YELLOW);

                    let dist_m = game.local_ship.position.distance_to(other_player.position) as u32;
                    let t_title = format!("🎯 {} • Lv.{} ({}m)", other_player.username, other_player.level, dist_m);
                    draw_crisp_text_shadow(custom_font.as_ref(), &t_title, tx + 12.0, ty + 20.0, 15.0, mocha::YELLOW);

                    let hp_p = (other_player.health / other_player.max_health).clamp(0.0, 1.0);
                    draw_rectangle(tx + 12.0, ty + 28.0, 266.0, 9.0, mocha::SURFACE0);
                    draw_rectangle(tx + 12.0, ty + 28.0, 266.0 * hp_p, 9.0, mocha::GREEN);

                    let sh_p = (other_player.shield / other_player.max_shield).clamp(0.0, 1.0);
                    draw_rectangle(tx + 12.0, ty + 41.0, 266.0, 7.0, mocha::SURFACE0);
                    draw_rectangle(tx + 12.0, ty + 41.0, 266.0 * sh_p, 7.0, mocha::SAPPHIRE);

                    let hp_txt = format!("{:.0} / {:.0} PV • {}", other_player.health, other_player.max_health, other_player.ship_class.name());
                    draw_crisp_text(custom_font.as_ref(), &hp_txt, tx + 12.0, ty + 60.0, 11.0, mocha::SKY);
                }
            }
        }

        // 16. Radar Minimap (Catppuccin Mocha)
        let radar_size = 145.0;
        let rx = screen_width() - radar_size - 18.0;
        let ry = screen_height() - radar_size - 18.0;

        draw_rectangle(rx, ry, radar_size, radar_size, Color::new(0.09, 0.09, 0.15, 0.90));
        draw_rectangle_lines(rx, ry, radar_size, radar_size, 1.5, mocha::SURFACE1);

        let radar_scale = radar_size / WORLD_WIDTH;
        let radar_center = Vec2::new(rx + radar_size * 0.5, ry + radar_size * 0.5);

        // Safe Base on Radar
        draw_circle_lines(radar_center.x, radar_center.y, game.space_base.radius * radar_scale, 1.0, mocha::SKY);

        // Minerals on Radar
        for m in &game.minerals {
            let mx = radar_center.x + m.position.x * radar_scale;
            let my = radar_center.y + m.position.y * radar_scale;
            draw_circle(mx, my, 1.2, mocha::SAPPHIRE);
        }

        // Aliens on Radar
        for a in &game.aliens {
            if a.health > 0.0 {
                let ax = radar_center.x + a.position.x * radar_scale;
                let ay = radar_center.y + a.position.y * radar_scale;
                let is_hostile = a.target_player_id == Some(game.local_player_id);
                let a_col = if a.alien_type == AlienType::Streuner && !is_hostile {
                    mocha::YELLOW
                } else {
                    mocha::RED
                };
                draw_circle(ax, ay, 2.0, a_col);

                if game.locked_target_id == Some(a.id) {
                    let ring_pulse = ((now * 10.0).sin() * 2.0 + 5.0).abs();
                    draw_circle_lines(ax, ay, ring_pulse, 1.5, mocha::YELLOW);
                }
            }
        }

        // Other Pilots on Radar
        for p in game.players.values() {
            if p.id != game.local_player_id && p.is_alive {
                let ox = radar_center.x + p.position.x * radar_scale;
                let oy = radar_center.y + p.position.y * radar_scale;
                draw_circle(ox, oy, 2.5, mocha::SKY);

                if game.locked_target_id == Some(p.id) {
                    let ring_pulse = ((now * 10.0).sin() * 2.0 + 5.0).abs();
                    draw_circle_lines(ox, oy, ring_pulse, 1.5, mocha::YELLOW);
                }
            }
        }

        // Loot Boxes on Radar
        for lb in &game.loot_boxes {
            let lx = radar_center.x + lb.position.x * radar_scale;
            let ly = radar_center.y + lb.position.y * radar_scale;
            draw_circle(lx, ly, 2.2, mocha::PEACH);
        }

        // Player on Radar
        let px = radar_center.x + game.local_ship.position.x * radar_scale;
        let py = radar_center.y + game.local_ship.position.y * radar_scale;
        draw_circle(px, py, 3.2, mocha::GREEN);

        // Radar Sweep Line
        let sweep_a = now * 2.2;
        draw_line(radar_center.x, radar_center.y, radar_center.x + sweep_a.cos() * (radar_size * 0.5), radar_center.y + sweep_a.sin() * (radar_size * 0.5), 1.0, Color::new(0.58, 0.89, 0.84, 0.4));

        // 17. Quick Commands Help & Shortcuts Box (Catppuccin Mocha)
        let q_w = 440.0;
        let q_h = 100.0;
        let q_x = 16.0;
        let q_y = screen_height() - q_h - 16.0;

        draw_rectangle(q_x, q_y, q_w, q_h, Color::new(0.09, 0.09, 0.15, 0.92));
        draw_rectangle_lines(q_x, q_y, q_w, q_h, 1.2, mocha::SURFACE1);

        draw_crisp_text_shadow(custom_font.as_ref(), "⚡ COMMANDES & RACCOURCIS ASTROBRAWL", q_x + 14.0, q_y + 22.0, 15.0, mocha::MAUVE);
        draw_crisp_text(custom_font.as_ref(), "• [Z Q S D] Déplacement direct  • [Espace / Clic G] Tirer", q_x + 14.0, q_y + 42.0, 13.0, mocha::TEXT);
        draw_crisp_text(custom_font.as_ref(), "• [TAB] Ciblage auto ennemi proche  • [Échap] Déverrouiller", q_x + 14.0, q_y + 60.0, 13.0, mocha::SAPPHIRE);
        draw_crisp_text(custom_font.as_ref(), "• [E] Laser Minier  • [H] Hangar  • [T] Talents  • [C] Craft  • [V] Vente", q_x + 14.0, q_y + 78.0, 13.0, mocha::PEACH);
        draw_crisp_text(custom_font.as_ref(), "• [J] Saut Portail  • [M] Muet / Activer le Son", q_x + 14.0, q_y + 94.0, 12.0, mocha::TEAL);

        // 18. Modals (Hangar, Talents, Crafting) - Catppuccin Mocha Overhaul
        match game.active_modal {
            ActiveModal::Hangar => {
                // Fullscreen Backdrop Dimming
                draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.07, 0.07, 0.11, 0.88));

                let mw = 760.0_f32.min(screen_width() - 40.0);
                let mh = 530.0_f32.min(screen_height() - 40.0);
                let mx = (screen_width() - mw) * 0.5;
                let my = (screen_height() - mh) * 0.5;

                // Modal Container Frame
                draw_rectangle(mx, my, mw, mh, mocha::BASE);
                draw_rectangle_lines(mx, my, mw, mh, 2.0, mocha::MAUVE);

                // Header Bar
                draw_rectangle(mx, my, mw, 54.0, mocha::MANTLE);
                draw_line(mx, my + 54.0, mx + mw, my + 54.0, 1.5, mocha::SURFACE1);
                draw_crisp_text_shadow(custom_font.as_ref(), "🚀 HANGAR SPATIAL • CHOIX DU VAISSEAU", mx + 22.0, my + 34.0, 21.0, mocha::TEXT);

                // Close Button in Header
                let close_btn_x = mx + mw - 100.0;
                let close_btn_y = my + 14.0;
                let close_hover = mouse_x >= close_btn_x && mouse_x <= close_btn_x + 85.0 && mouse_y >= close_btn_y && mouse_y <= close_btn_y + 26.0;
                draw_rectangle(close_btn_x, close_btn_y, 85.0, 26.0, if close_hover { mocha::SURFACE1 } else { mocha::SURFACE0 });
                draw_rectangle_lines(close_btn_x, close_btn_y, 85.0, 26.0, 1.0, if close_hover { mocha::RED } else { mocha::SURFACE2 });
                draw_crisp_text(custom_font.as_ref(), "✕ ÉCHAP", close_btn_x + 18.0, close_btn_y + 18.0, 13.0, mocha::TEXT);
                if close_hover && mouse_clicked {
                    game.active_modal = ActiveModal::None;
                    audio::play(audio::SFX_UI_CLICK);
                }

                let classes = [
                    (KeyCode::Key1, "1", ShipClass::Combat, "1. Intercepteur de Combat", "Canons surchargés, cadence soutenue et grande agilité de combat. Non équipé pour le minage.", "⚡ Vitesse: 380 px/s  •  💥 Dégâts: +32 (Laser Pulsé)  •  🛡️ Bouclier: 150  •  📦 Soute: 100 kg", mocha::BLUE),
                    (KeyCode::Key2, "2", ShipClass::Minier, "2. Extracteur Minier", "Équipé du Laser de Forage thermique rotatif. Seule classe capable de récolter les minerais spatiaux.", "⚡ Vitesse: 310 px/s  •  ⛏️ Laser de Forage  •  🛡️ Bouclier: 180  •  📦 Soute: 300 kg", mocha::YELLOW),
                    (KeyCode::Key3, "3", ShipClass::Transport, "3. Mastodonte Cargo", "Blindage composite lourd et coque renforcée. Conçu pour le transport massif et le commerce.", "⚡ Vitesse: 240 px/s  •  🛡️ Bouclier: 320  •  ❤️ HP: 360  •  📦 Soute Titanesque: 1000 kg", mocha::PEACH),
                    (KeyCode::Key4, "4", ShipClass::Exploration, "4. Éclaireur Longue Portée", "Propulseur à impulsion avancée. Vitesse suprême pour cartographier les secteurs et failles cosmiques.", "⚡ Vitesse: 480 px/s  •  📡 Radar Étendu  •  🛡️ Bouclier: 160  •  📦 Soute: 150 kg", mocha::GREEN),
                ];

                for (idx, (key, key_str, class, title, desc, stats, accent_col)) in classes.iter().enumerate() {
                    let card_y = my + 68.0 + (idx as f32 * 105.0);
                    let card_h = 94.0;
                    let card_w = mw - 40.0;
                    let card_x = mx + 20.0;

                    let is_active = game.local_ship.ship_class == *class;
                    let is_hovered = mouse_x >= card_x && mouse_x <= card_x + card_w && mouse_y >= card_y && mouse_y <= card_y + card_h;

                    let bg_col = if is_active {
                        Color::new(0.19, 0.20, 0.27, 0.95) // surface0
                    } else if is_hovered {
                        Color::new(0.15, 0.15, 0.22, 0.95)
                    } else {
                        Color::new(0.09, 0.09, 0.15, 0.90) // mantle
                    };

                    draw_rectangle(card_x, card_y, card_w, card_h, bg_col);
                    draw_rectangle_lines(card_x, card_y, card_w, card_h, if is_active { 2.0 } else { 1.0 }, if is_active { mocha::GREEN } else if is_hovered { *accent_col } else { mocha::SURFACE0 });

                    // Left Accent Stripe
                    draw_rectangle(card_x, card_y, 6.0, card_h, *accent_col);

                    // Key Badge Pill [ 1 ]
                    let kb_x = card_x + 18.0;
                    let kb_y = card_y + 14.0;
                    draw_rectangle(kb_x, kb_y, 34.0, 26.0, mocha::CRUST);
                    draw_rectangle_lines(kb_x, kb_y, 34.0, 26.0, 1.0, mocha::SURFACE2);
                    draw_crisp_text(custom_font.as_ref(), key_str, kb_x + 12.0, kb_y + 18.0, 15.0, mocha::TEXT);

                    // Title
                    draw_crisp_text_shadow(custom_font.as_ref(), title, card_x + 62.0, card_y + 26.0, 17.0, if is_active { mocha::GREEN } else { mocha::TEXT });

                    // Equipped status / select button
                    if is_active {
                        let eq_w = 90.0;
                        let eq_x = card_x + card_w - eq_w - 16.0;
                        draw_rectangle(eq_x, card_y + 14.0, eq_w, 24.0, Color::new(0.65, 0.89, 0.63, 0.2));
                        draw_rectangle_lines(eq_x, card_y + 14.0, eq_w, 24.0, 1.0, mocha::GREEN);
                        draw_crisp_text(custom_font.as_ref(), "✓ ÉQUIPÉ", eq_x + 18.0, card_y + 30.0, 13.0, mocha::GREEN);
                    } else {
                        let sel_w = 115.0;
                        let sel_x = card_x + card_w - sel_w - 16.0;
                        draw_rectangle(sel_x, card_y + 14.0, sel_w, 24.0, mocha::SURFACE0);
                        draw_rectangle_lines(sel_x, card_y + 14.0, sel_w, 24.0, 1.0, if is_hovered { *accent_col } else { mocha::SURFACE1 });
                        draw_crisp_text(custom_font.as_ref(), "CHOISIR [Clic]", sel_x + 16.0, card_y + 30.0, 12.0, mocha::SUBTEXT1);
                    }

                    // Description & Stats
                    draw_crisp_text(custom_font.as_ref(), desc, card_x + 22.0, card_y + 54.0, 14.0, mocha::SUBTEXT1);
                    draw_crisp_text(custom_font.as_ref(), stats, card_x + 22.0, card_y + 78.0, 13.0, *accent_col);

                    if (is_key_pressed(*key) || (is_hovered && mouse_clicked)) && !is_active {
                        audio::play(audio::SFX_UI_CLICK);
                        game.local_ship.ship_class = *class;
                        game.local_ship.max_health = class.base_health();
                        game.local_ship.health = class.base_health();
                        game.local_ship.max_shield = class.base_shield();
                        game.local_ship.shield = class.base_shield();
                        game.local_ship.apply_talent_bonuses();
                        game.spawn_float_text(format!("🚀 Vaisseau équipé : {}", class.name()), game.local_ship.position, mocha::GREEN);
                        game.save_progression();
                        game.send_message(&ClientMessage::SelectClass { class: *class });
                    }
                }

                draw_crisp_text(custom_font.as_ref(), "Appuyez sur [1, 2, 3, 4], cliquez sur un vaisseau ou appuyez sur [Échap] pour fermer", mx + 110.0, my + mh - 16.0, 13.0, mocha::OVERLAY1);
            }
            ActiveModal::Talents => {
                // Fullscreen Backdrop Dimming
                draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.07, 0.07, 0.11, 0.88));

                let mw = 760.0_f32.min(screen_width() - 40.0);
                let mh = 530.0_f32.min(screen_height() - 40.0);
                let mx = (screen_width() - mw) * 0.5;
                let my = (screen_height() - mh) * 0.5;

                draw_rectangle(mx, my, mw, mh, mocha::BASE);
                draw_rectangle_lines(mx, my, mw, mh, 2.0, mocha::GREEN);

                // Header Bar
                draw_rectangle(mx, my, mw, 54.0, mocha::MANTLE);
                draw_line(mx, my + 54.0, mx + mw, my + 54.0, 1.5, mocha::SURFACE1);
                draw_crisp_text_shadow(custom_font.as_ref(), "🧬 ARBRE DE TALENTS DU COMMANDANT", mx + 22.0, my + 34.0, 21.0, mocha::TEXT);

                let pts_str = format!("⭐ Points disponibles : {}", game.local_ship.talent_points);
                draw_crisp_text_shadow(custom_font.as_ref(), &pts_str, mx + 380.0, my + 34.0, 16.0, mocha::YELLOW);

                // Close Button in Header
                let close_btn_x = mx + mw - 100.0;
                let close_btn_y = my + 14.0;
                let close_hover = mouse_x >= close_btn_x && mouse_x <= close_btn_x + 85.0 && mouse_y >= close_btn_y && mouse_y <= close_btn_y + 26.0;
                draw_rectangle(close_btn_x, close_btn_y, 85.0, 26.0, if close_hover { mocha::SURFACE1 } else { mocha::SURFACE0 });
                draw_rectangle_lines(close_btn_x, close_btn_y, 85.0, 26.0, 1.0, if close_hover { mocha::RED } else { mocha::SURFACE2 });
                draw_crisp_text(custom_font.as_ref(), "✕ ÉCHAP", close_btn_x + 18.0, close_btn_y + 18.0, 13.0, mocha::TEXT);
                if close_hover && mouse_clicked {
                    game.active_modal = ActiveModal::None;
                    audio::play(audio::SFX_UI_CLICK);
                }

                let talents_data = [
                    (KeyCode::Key1, "1", 0, "Optique Laser Amplifiée", "+6% Dégâts laser supplémentaires par rang", game.local_ship.talents.combat_laser_dmg),
                    (KeyCode::Key2, "2", 1, "Surchauffeur de Flux", "+5% Cadence de tir soutenue par rang", game.local_ship.talents.combat_fire_rate),
                    (KeyCode::Key3, "3", 2, "Condensateur de Bouclier", "+10% Capacité maximale de bouclier par rang", game.local_ship.talents.defense_shield_max),
                    (KeyCode::Key4, "4", 3, "Nano-Réparateur de Bord", "+15% Vitesse de régénération passive du bouclier", game.local_ship.talents.defense_regen),
                    (KeyCode::Key5, "5", 4, "Soute Compressée Quantum", "+20% Capacité maximale de la soute en kg", game.local_ship.talents.logistics_cargo),
                    (KeyCode::Key6, "6", 5, "Foreuse Thermique Avancée", "+25% Vitesse d'extraction des minerais", game.local_ship.talents.logistics_mining_speed),
                ];

                for (idx, (key, key_str, talent_idx, title, desc, level)) in talents_data.iter().enumerate() {
                    let card_y = my + 68.0 + (idx as f32 * 68.0);
                    let card_h = 58.0;
                    let card_w = mw - 40.0;
                    let card_x = mx + 20.0;

                    let can_upgrade = game.local_ship.talent_points > 0 && *level < 5;
                    let is_max = *level >= 5;
                    let is_hovered = mouse_x >= card_x && mouse_x <= card_x + card_w && mouse_y >= card_y && mouse_y <= card_y + card_h;

                    let bg_col = if is_hovered && can_upgrade {
                        Color::new(0.18, 0.22, 0.30, 0.95)
                    } else {
                        Color::new(0.09, 0.09, 0.15, 0.90)
                    };

                    draw_rectangle(card_x, card_y, card_w, card_h, bg_col);
                    draw_rectangle_lines(card_x, card_y, card_w, card_h, 1.0, if can_upgrade && is_hovered { mocha::GREEN } else { mocha::SURFACE0 });

                    // Key Badge Pill
                    let kb_x = card_x + 14.0;
                    let kb_y = card_y + 16.0;
                    draw_rectangle(kb_x, kb_y, 30.0, 26.0, mocha::CRUST);
                    draw_rectangle_lines(kb_x, kb_y, 30.0, 26.0, 1.0, mocha::SURFACE2);
                    draw_crisp_text(custom_font.as_ref(), key_str, kb_x + 10.0, kb_y + 18.0, 15.0, mocha::TEXT);

                    // Title & Description
                    draw_crisp_text_shadow(custom_font.as_ref(), title, card_x + 56.0, card_y + 24.0, 16.0, mocha::TEXT);
                    draw_crisp_text(custom_font.as_ref(), desc, card_x + 56.0, card_y + 44.0, 13.0, mocha::SUBTEXT1);

                    // 5 Level Pips
                    let pips_x = card_x + card_w - 200.0;
                    for p in 0..5 {
                        let pip_cx = pips_x + (p as f32 * 14.0);
                        let pip_cy = card_y + 28.0;
                        if (p as u32) < *level {
                            draw_circle(pip_cx, pip_cy, 5.0, mocha::GREEN);
                        } else {
                            draw_circle_lines(pip_cx, pip_cy, 5.0, 1.2, mocha::SURFACE2);
                        }
                    }

                    let lvl_str = format!("{} / 5", level);
                    draw_crisp_text(custom_font.as_ref(), &lvl_str, pips_x + 78.0, card_y + 32.0, 13.0, mocha::LAVENDER);

                    // Upgrade Button
                    let up_w = 90.0;
                    let up_x = card_x + card_w - up_w - 14.0;
                    let up_y = card_y + 14.0;

                    if is_max {
                        draw_rectangle(up_x, up_y, up_w, 28.0, mocha::SURFACE0);
                        draw_crisp_text(custom_font.as_ref(), "MAX", up_x + 30.0, up_y + 19.0, 13.0, mocha::PEACH);
                    } else if can_upgrade {
                        draw_rectangle(up_x, up_y, up_w, 28.0, Color::new(0.65, 0.89, 0.63, 0.2));
                        draw_rectangle_lines(up_x, up_y, up_w, 28.0, 1.2, mocha::GREEN);
                        draw_crisp_text(custom_font.as_ref(), "+1 [Clic]", up_x + 18.0, up_y + 19.0, 13.0, mocha::GREEN);
                    } else {
                        draw_rectangle(up_x, up_y, up_w, 28.0, mocha::CRUST);
                        draw_crisp_text(custom_font.as_ref(), "+1", up_x + 38.0, up_y + 19.0, 13.0, mocha::OVERLAY0);
                    }

                    if (is_key_pressed(*key) || (is_hovered && mouse_clicked)) && can_upgrade {
                        audio::play(audio::SFX_UI_CLICK);
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
                        game.spawn_float_text(format!("🧬 Compétence améliorée : {}", title), game.local_ship.position, mocha::GREEN);
                        game.save_progression();
                        game.send_message(&ClientMessage::UpgradeTalent { talent_index: *talent_idx });
                    }
                }

                draw_crisp_text(custom_font.as_ref(), "Appuyez sur [1 à 6] ou cliquez pour dépenser vos points • [Échap] pour fermer", mx + 110.0, my + mh - 16.0, 13.0, mocha::OVERLAY1);
            }
            ActiveModal::Crafting => {
                // Fullscreen Backdrop Dimming
                draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.07, 0.07, 0.11, 0.88));

                let mw = 760.0_f32.min(screen_width() - 40.0);
                let mh = 540.0_f32.min(screen_height() - 40.0);
                let mx = (screen_width() - mw) * 0.5;
                let my = (screen_height() - mh) * 0.5;

                draw_rectangle(mx, my, mw, mh, mocha::BASE);
                draw_rectangle_lines(mx, my, mw, mh, 2.0, mocha::YELLOW);

                // Header Bar
                draw_rectangle(mx, my, mw, 54.0, mocha::MANTLE);
                draw_line(mx, my + 54.0, mx + mw, my + 54.0, 1.5, mocha::SURFACE1);
                draw_crisp_text_shadow(custom_font.as_ref(), "🛠️ ATELIER DE FABRICATION SPATIALE", mx + 22.0, my + 34.0, 21.0, mocha::TEXT);

                // Close Button in Header
                let close_btn_x = mx + mw - 100.0;
                let close_btn_y = my + 14.0;
                let close_hover = mouse_x >= close_btn_x && mouse_x <= close_btn_x + 85.0 && mouse_y >= close_btn_y && mouse_y <= close_btn_y + 26.0;
                draw_rectangle(close_btn_x, close_btn_y, 85.0, 26.0, if close_hover { mocha::SURFACE1 } else { mocha::SURFACE0 });
                draw_rectangle_lines(close_btn_x, close_btn_y, 85.0, 26.0, 1.0, if close_hover { mocha::RED } else { mocha::SURFACE2 });
                draw_crisp_text(custom_font.as_ref(), "✕ ÉCHAP", close_btn_x + 18.0, close_btn_y + 18.0, 13.0, mocha::TEXT);
                if close_hover && mouse_clicked {
                    game.active_modal = ActiveModal::None;
                    audio::play(audio::SFX_UI_CLICK);
                }

                // Resource Pill Bar
                let res_y = my + 64.0;
                let res_bar = format!("💎 Prom: {}  •  End: {}  •  Terb: {}  •  Ferraille: {}  •  Crédits: {} C",
                    game.local_ship.cargo.prometium, game.local_ship.cargo.endurium, game.local_ship.cargo.terbium,
                    game.local_ship.cargo.scrap, game.local_ship.credits);
                draw_crisp_text_shadow(custom_font.as_ref(), &res_bar, mx + 24.0, res_y + 14.0, 14.0, mocha::SAPPHIRE);

                let keys = [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Key4];
                let key_names = ["1", "2", "3", "4"];

                for (idx, recipe) in CRAFT_RECIPES.iter().enumerate() {
                    let card_y = my + 94.0 + (idx as f32 * 96.0);
                    let card_h = 86.0;
                    let card_w = mw - 40.0;
                    let card_x = mx + 20.0;

                    let can_craft = game.local_ship.cargo.prometium >= recipe.cost_prometium
                        && game.local_ship.cargo.endurium >= recipe.cost_endurium
                        && game.local_ship.cargo.terbium >= recipe.cost_terbium
                        && game.local_ship.cargo.scrap >= recipe.cost_scrap
                        && game.local_ship.credits >= recipe.cost_credits;

                    let is_hovered = mouse_x >= card_x && mouse_x <= card_x + card_w && mouse_y >= card_y && mouse_y <= card_y + card_h;

                    let bg_col = if is_hovered && can_craft {
                        Color::new(0.18, 0.22, 0.30, 0.95)
                    } else {
                        Color::new(0.09, 0.09, 0.15, 0.90)
                    };

                    draw_rectangle(card_x, card_y, card_w, card_h, bg_col);
                    draw_rectangle_lines(card_x, card_y, card_w, card_h, 1.0, if can_craft { mocha::GREEN } else { mocha::SURFACE0 });

                    // Key Badge Pill
                    let kb_x = card_x + 14.0;
                    let kb_y = card_y + 14.0;
                    draw_rectangle(kb_x, kb_y, 30.0, 26.0, mocha::CRUST);
                    draw_rectangle_lines(kb_x, kb_y, 30.0, 26.0, 1.0, mocha::SURFACE2);
                    draw_crisp_text(custom_font.as_ref(), key_names[idx], kb_x + 10.0, kb_y + 18.0, 15.0, mocha::TEXT);

                    // Recipe Title & Description
                    draw_crisp_text_shadow(custom_font.as_ref(), recipe.name, card_x + 56.0, card_y + 24.0, 17.0, if can_craft { mocha::YELLOW } else { mocha::TEXT });
                    draw_crisp_text(custom_font.as_ref(), recipe.description, card_x + 56.0, card_y + 44.0, 13.0, mocha::SUBTEXT1);

                    // Required Resources line
                    let cost_desc = format!("Coût : {} Prom, {} End, {} Terb, {} Ferraille, {} C",
                        recipe.cost_prometium, recipe.cost_endurium, recipe.cost_terbium, recipe.cost_scrap, recipe.cost_credits);
                    draw_crisp_text(custom_font.as_ref(), &cost_desc, card_x + 56.0, card_y + 68.0, 12.0, if can_craft { mocha::GREEN } else { mocha::RED });

                    // Craft Button
                    let cb_w = 120.0;
                    let cb_x = card_x + card_w - cb_w - 14.0;
                    let cb_y = card_y + 16.0;

                    if can_craft {
                        draw_rectangle(cb_x, cb_y, cb_w, 28.0, Color::new(0.65, 0.89, 0.63, 0.2));
                        draw_rectangle_lines(cb_x, cb_y, cb_w, 28.0, 1.2, mocha::GREEN);
                        draw_crisp_text(custom_font.as_ref(), "ASSEMBLER [Clic]", cb_x + 14.0, cb_y + 19.0, 12.0, mocha::GREEN);
                    } else {
                        draw_rectangle(cb_x, cb_y, cb_w, 28.0, mocha::CRUST);
                        draw_crisp_text(custom_font.as_ref(), "MANQUE RESSOURCES", cb_x + 8.0, cb_y + 19.0, 10.0, mocha::OVERLAY0);
                    }

                    if (is_key_pressed(keys[idx]) || (is_hovered && mouse_clicked)) && can_craft {
                        audio::play(audio::SFX_CRAFT);
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

                        game.spawn_float_text(format!("✨ Assemblé avec succès : {}", recipe.name), game.local_ship.position, mocha::YELLOW);
                        game.save_progression();
                        game.send_message(&ClientMessage::Craft { recipe_index: idx as u32 });
                    }
                }

                draw_crisp_text(custom_font.as_ref(), "Appuyez sur [1 à 4] ou cliquez sur une recette pour l'assembler • [Échap] pour fermer", mx + 100.0, my + mh - 16.0, 13.0, mocha::OVERLAY1);
            }
            ActiveModal::None => {}
        }

        // 19. Respawn Overlay (Catppuccin Mocha)
        if !game.local_ship.is_alive {
            draw_rectangle(0.0, screen_height() * 0.35, screen_width(), 130.0, Color::new(0.07, 0.07, 0.11, 0.94));
            let d_title = "⚡ VAISSEAU DÉTRUIT ⚡";
            let dw = measure_crisp_text(custom_font.as_ref(), d_title, 34.0).width;
            draw_crisp_text_shadow(custom_font.as_ref(), d_title, (screen_width() - dw) * 0.5, screen_height() * 0.43, 34.0, mocha::RED);

            let sub = "Appuyez sur ESPACE ou R pour vous réincarner à la Base Spatiale";
            let sw = measure_crisp_text(custom_font.as_ref(), sub, 18.0).width;
            draw_crisp_text_shadow(custom_font.as_ref(), sub, (screen_width() - sw) * 0.5, screen_height() * 0.49, 18.0, mocha::TEXT);
        }

        next_frame().await;
    }
}

