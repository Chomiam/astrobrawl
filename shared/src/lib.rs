use serde::{Deserialize, Serialize};

// --- Game Constants ---
pub const TICK_RATE: u64 = 30;
pub const TICK_INTERVAL_MS: u64 = 1000 / TICK_RATE;
pub const TICK_DT: f32 = 1.0 / (TICK_RATE as f32);

pub const WORLD_WIDTH: f32 = 3600.0;
pub const WORLD_HEIGHT: f32 = 3600.0;

pub const SHIP_RADIUS: f32 = 22.0;
pub const SHIP_MAX_SPEED: f32 = 360.0;
pub const SHIP_ACCELERATION: f32 = 520.0;
pub const SHIP_DAMPING: f32 = 0.985;
pub const SHIP_TURN_SPEED: f32 = 8.0;

pub const SHIP_BASE_HEALTH: f32 = 100.0;
pub const SHIP_BASE_SHIELD: f32 = 100.0;
pub const SHIELD_REGEN_PER_SEC: f32 = 6.0;

pub const LASER_SPEED: f32 = 750.0;
pub const LASER_LIFETIME: f32 = 1.1;
pub const LASER_DAMAGE: f32 = 25.0;
pub const LASER_COOLDOWN: f32 = 0.22;

pub const MINERAL_RADIUS: f32 = 14.0;
pub const MINERAL_COLLECT_RADIUS: f32 = 42.0;
pub const MAX_MINERALS_ON_MAP: usize = 65;

// --- Vector Math ---
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn length_squared(self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    pub fn length(self) -> f32 {
        self.length_squared().sqrt()
    }

    pub fn distance_to(self, other: Self) -> f32 {
        (self - other).length()
    }

    pub fn normalize(self) -> Self {
        let len = self.length();
        if len > 0.0001 {
            Self {
                x: self.x / len,
                y: self.y / len,
            }
        } else {
            Self::ZERO
        }
    }

    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
        }
    }
}

impl std::ops::Add for Vec2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::Sub for Vec2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl std::ops::Mul<f32> for Vec2 {
    type Output = Self;
    fn mul(self, scalar: f32) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

impl std::ops::AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

// --- Player & Ship Types ---
pub type PlayerId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShipModel {
    Phoenix,
    Yamato,
    Goliath,
}

impl Default for ShipModel {
    fn default() -> Self {
        Self::Phoenix
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerShip {
    pub id: PlayerId,
    pub username: String,
    pub model: ShipModel,
    pub position: Vec2,
    pub velocity: Vec2,
    pub rotation: f32,
    pub health: f32,
    pub max_health: f32,
    pub shield: f32,
    pub max_shield: f32,
    pub credits: u32,
    pub minerals: u32,
    pub score: u32,
    pub is_thrusting: bool,
    pub is_alive: bool,
}

impl PlayerShip {
    pub fn new(id: PlayerId, username: String, position: Vec2) -> Self {
        Self {
            id,
            username,
            model: ShipModel::Phoenix,
            position,
            velocity: Vec2::ZERO,
            rotation: 0.0,
            health: SHIP_BASE_HEALTH,
            max_health: SHIP_BASE_HEALTH,
            shield: SHIP_BASE_SHIELD,
            max_shield: SHIP_BASE_SHIELD,
            credits: 1000,
            minerals: 0,
            score: 0,
            is_thrusting: false,
            is_alive: true,
        }
    }
}

// --- Projectiles ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Laser {
    pub id: u64,
    pub shooter_id: PlayerId,
    pub position: Vec2,
    pub velocity: Vec2,
    pub lifetime: f32,
}

// --- Minerals ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MineralType {
    Prometium, // Orange / Red
    Endurium,  // Cyan / Blue
    Terbium,   // Emerald / Yellow-Green
}

impl MineralType {
    pub fn color_rgba(&self) -> [f32; 4] {
        match self {
            Self::Prometium => [1.0, 0.42, 0.15, 1.0],
            Self::Endurium => [0.15, 0.82, 1.0, 1.0],
            Self::Terbium => [0.25, 0.95, 0.35, 1.0],
        }
    }

    pub fn value(&self) -> u32 {
        match self {
            Self::Prometium => 10,
            Self::Endurium => 25,
            Self::Terbium => 50,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mineral {
    pub id: u64,
    pub mineral_type: MineralType,
    pub position: Vec2,
    pub value: u32,
}

// --- World Snapshot ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSnapshot {
    pub tick: u64,
    pub server_time_ms: u64,
    pub players: Vec<PlayerShip>,
    pub lasers: Vec<Laser>,
    pub minerals: Vec<Mineral>,
}

// --- Network Messages ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    Auth { token: String },
    Input {
        thrust: bool,
        target_angle: f32,
    },
    Shoot,
    Ping { client_time: u64 },
    Respawn,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    AuthSuccess {
        player_id: PlayerId,
        username: String,
        ship: PlayerShip,
    },
    AuthError {
        message: String,
    },
    WorldSnapshot(WorldSnapshot),
    PlayerJoined {
        id: PlayerId,
        username: String,
    },
    PlayerLeft {
        id: PlayerId,
    },
    PlayerKilled {
        victim_id: PlayerId,
        killer_id: PlayerId,
    },
    MineralCollected {
        player_id: PlayerId,
        mineral_id: u64,
        mineral_type: MineralType,
        value: u32,
        total_minerals: u32,
        total_credits: u32,
    },
    StatsUpdated {
        health: f32,
        shield: f32,
        minerals: u32,
        credits: u32,
        score: u32,
    },
    Pong {
        client_time: u64,
        server_time: u64,
    },
}

// --- Bincode Helpers ---
pub fn serialize_packet<T: Serialize>(packet: &T) -> Result<Vec<u8>, String> {
    bincode::serde::encode_to_vec(packet, bincode::config::standard())
        .map_err(|e| format!("Serialization error: {}", e))
}

pub fn deserialize_packet<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, String> {
    bincode::serde::decode_from_slice(bytes, bincode::config::standard())
        .map(|(val, _)| val)
        .map_err(|e| format!("Deserialization error: {}", e))
}

// --- Inertial Physics Helper (Shared between Client Prediction & Server Authoritative loop) ---
pub fn apply_ship_physics(ship: &mut PlayerShip, thrust: bool, target_angle: f32, dt: f32) {
    if !ship.is_alive {
        ship.velocity = Vec2::ZERO;
        ship.is_thrusting = false;
        return;
    }

    let mut diff = target_angle - ship.rotation;
    while diff < -std::f32::consts::PI {
        diff += std::f32::consts::TAU;
    }
    while diff > std::f32::consts::PI {
        diff -= std::f32::consts::TAU;
    }

    let max_rot_step = SHIP_TURN_SPEED * dt;
    if diff.abs() <= max_rot_step {
        ship.rotation = target_angle;
    } else {
        ship.rotation += diff.signum() * max_rot_step;
    }

    ship.is_thrusting = thrust;
    if thrust {
        let thrust_dir = Vec2::new(ship.rotation.cos(), ship.rotation.sin());
        ship.velocity += thrust_dir * (SHIP_ACCELERATION * dt);

        let speed = ship.velocity.length();
        if speed > SHIP_MAX_SPEED {
            ship.velocity = ship.velocity.normalize() * SHIP_MAX_SPEED;
        }
    }

    let damping = SHIP_DAMPING.powf(dt * 60.0);
    ship.velocity = ship.velocity * damping;

    ship.position += ship.velocity * dt;

    let half_w = WORLD_WIDTH * 0.5;
    let half_h = WORLD_HEIGHT * 0.5;

    if ship.position.x < -half_w {
        ship.position.x = -half_w;
        ship.velocity.x = -ship.velocity.x * 0.5;
    } else if ship.position.x > half_w {
        ship.position.x = half_w;
        ship.velocity.x = -ship.velocity.x * 0.5;
    }

    if ship.position.y < -half_h {
        ship.position.y = -half_h;
        ship.velocity.y = -ship.velocity.y * 0.5;
    } else if ship.position.y > half_h {
        ship.position.y = half_h;
        ship.velocity.y = -ship.velocity.y * 0.5;
    }
}
