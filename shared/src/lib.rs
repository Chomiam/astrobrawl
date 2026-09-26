use serde::{Deserialize, Serialize};

// --- Game Constants ---
pub const TICK_RATE: u64 = 30;
pub const TICK_INTERVAL_MS: u64 = 1000 / TICK_RATE;
pub const TICK_DT: f32 = 1.0 / (TICK_RATE as f32);

pub const WORLD_WIDTH: f32 = 4200.0;
pub const WORLD_HEIGHT: f32 = 4200.0;

pub const SHIP_RADIUS: f32 = 22.0;
pub const SHIP_DAMPING: f32 = 0.985;
pub const SHIP_TURN_SPEED: f32 = 7.5;

pub const LASER_SPEED: f32 = 800.0;
pub const LASER_LIFETIME: f32 = 1.05;

pub const MINING_RANGE: f32 = 240.0;
pub const MINING_DAMAGE_PER_SEC: f32 = 45.0;

pub const MINERAL_RADIUS: f32 = 16.0;
pub const LOOTBOX_RADIUS: f32 = 18.0;
pub const BASE_RADIUS: f32 = 320.0;

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

    pub fn clamp_length_max(self, max: f32) -> Self {
        let len_sq = self.length_squared();
        if len_sq > max * max && len_sq > 0.0001 {
            let len = len_sq.sqrt();
            Self {
                x: self.x * (max / len),
                y: self.y * (max / len),
            }
        } else {
            self
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

impl std::ops::Neg for Vec2 {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
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

impl std::ops::MulAssign<f32> for Vec2 {
    fn mul_assign(&mut self, scalar: f32) {
        self.x *= scalar;
        self.y *= scalar;
    }
}

impl std::ops::AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

// --- Ship Classes ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShipClass {
    Combat,      // Chasseur / Intercepteur : rapide, puissants lasers, soute faible, ne mine pas
    Minier,      // Extracteur : laser de forage exclusif, grande soute, rendement bonus
    Transport,   // Mastodonte : soute colossale, blindage massif, vitesse lente
    Exploration, // Éclaireur : vitesse suprême, radar x2.5, détection d'anomalies
}

impl ShipClass {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Combat => "Intercepteur de Combat",
            Self::Minier => "Extracteur Minier",
            Self::Transport => "Mastodonte Cargo",
            Self::Exploration => "Éclaireur Longue Portée",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Combat => "Canons lasers surchargés, haute agilité. Ne peut pas extraire les minerais bruts.",
            Self::Minier => "Équipé du Laser de Forage thermique. Seul vaisseau capable d'extraire les minerais.",
            Self::Transport => "Soute de 1000 kg et bouclier titane lourd. Vitesse réduite mais protection maximale.",
            Self::Exploration => "Vitesse extrême et radar étendu x2.5 pour révéler les failles cosmiques secrètes.",
        }
    }

    pub fn base_speed(&self) -> f32 {
        match self {
            Self::Combat => 410.0,
            Self::Minier => 300.0,
            Self::Transport => 230.0,
            Self::Exploration => 480.0,
        }
    }

    pub fn base_acceleration(&self) -> f32 {
        match self {
            Self::Combat => 620.0,
            Self::Minier => 480.0,
            Self::Transport => 340.0,
            Self::Exploration => 740.0,
        }
    }

    pub fn base_health(&self) -> f32 {
        match self {
            Self::Combat => 100.0,
            Self::Minier => 130.0,
            Self::Transport => 260.0,
            Self::Exploration => 85.0,
        }
    }

    pub fn base_shield(&self) -> f32 {
        match self {
            Self::Combat => 120.0,
            Self::Minier => 90.0,
            Self::Transport => 240.0,
            Self::Exploration => 75.0,
        }
    }

    pub fn base_cargo(&self) -> u32 {
        match self {
            Self::Combat => 60,
            Self::Minier => 300,
            Self::Transport => 1000,
            Self::Exploration => 140,
        }
    }

    pub fn can_mine(&self) -> bool {
        matches!(self, Self::Minier)
    }

    pub fn base_laser_damage(&self) -> f32 {
        match self {
            Self::Combat => 32.0,
            Self::Minier => 15.0,
            Self::Transport => 22.0,
            Self::Exploration => 18.0,
        }
    }

    pub fn laser_cooldown(&self) -> f32 {
        match self {
            Self::Combat => 0.18,
            Self::Minier => 0.28,
            Self::Transport => 0.32,
            Self::Exploration => 0.22,
        }
    }
}

// --- Cargo System ---
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CargoHold {
    pub prometium: u32,
    pub endurium: u32,
    pub terbium: u32,
    pub seprom: u32,
    pub scrap: u32,
    pub plasma_cores: u32,
    pub max_capacity: u32,
}

impl CargoHold {
    pub fn new(capacity: u32) -> Self {
        Self {
            prometium: 0,
            endurium: 0,
            terbium: 0,
            seprom: 0,
            scrap: 0,
            plasma_cores: 0,
            max_capacity: capacity,
        }
    }

    pub fn used_capacity(&self) -> u32 {
        self.prometium
            + self.endurium * 2
            + self.terbium * 3
            + self.seprom * 5
            + self.scrap
            + self.plasma_cores * 2
    }

    pub fn free_space(&self) -> u32 {
        self.max_capacity.saturating_sub(self.used_capacity())
    }

    pub fn is_full(&self) -> bool {
        self.used_capacity() >= self.max_capacity
    }

    pub fn add_mineral(&mut self, m: MineralType, amount: u32) -> bool {
        let weight = m.weight() * amount;
        if self.free_space() >= weight {
            match m {
                MineralType::Prometium => self.prometium += amount,
                MineralType::Endurium => self.endurium += amount,
                MineralType::Terbium => self.terbium += amount,
                MineralType::Seprom => self.seprom += amount,
            }
            true
        } else {
            false
        }
    }

    pub fn add_scrap(&mut self, amount: u32) -> bool {
        if self.free_space() >= amount {
            self.scrap += amount;
            true
        } else {
            false
        }
    }

    pub fn add_plasma_core(&mut self, amount: u32) -> bool {
        if self.free_space() >= amount * 2 {
            self.plasma_cores += amount;
            true
        } else {
            false
        }
    }

    pub fn total_value_credits(&self) -> u32 {
        self.prometium * MineralType::Prometium.value()
            + self.endurium * MineralType::Endurium.value()
            + self.terbium * MineralType::Terbium.value()
            + self.seprom * MineralType::Seprom.value()
    }

    pub fn sell_all_minerals(&mut self) -> u32 {
        let credits = self.total_value_credits();
        self.prometium = 0;
        self.endurium = 0;
        self.terbium = 0;
        self.seprom = 0;
        credits
    }
}

// --- Minerals & Rarity ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MineralType {
    Prometium, // Commun - Orange
    Endurium,  // Peu commun - Bleu Cyan
    Terbium,   // Rare - Vert Émeraude
    Seprom,    // Exotique / Ultra-Rare - Violet Prismatique
}

impl MineralType {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Prometium => "Prometium",
            Self::Endurium => "Endurium",
            Self::Terbium => "Terbium",
            Self::Seprom => "Seprom",
        }
    }

    pub fn value(&self) -> u32 {
        match self {
            Self::Prometium => 12,
            Self::Endurium => 30,
            Self::Terbium => 75,
            Self::Seprom => 240,
        }
    }

    pub fn weight(&self) -> u32 {
        match self {
            Self::Prometium => 1,
            Self::Endurium => 2,
            Self::Terbium => 3,
            Self::Seprom => 5,
        }
    }

    pub fn max_health(&self) -> f32 {
        match self {
            Self::Prometium => 45.0,
            Self::Endurium => 80.0,
            Self::Terbium => 135.0,
            Self::Seprom => 250.0,
        }
    }

    pub fn color_rgba(&self) -> [f32; 4] {
        match self {
            Self::Prometium => [1.0, 0.45, 0.1, 1.0],  // Orange ardent
            Self::Endurium => [0.1, 0.85, 1.0, 1.0],   // Cyan néon
            Self::Terbium => [0.2, 1.0, 0.4, 1.0],     // Vert émeraude
            Self::Seprom => [0.85, 0.25, 1.0, 1.0],    // Violet cosmique
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mineral {
    pub id: u64,
    pub mineral_type: MineralType,
    pub position: Vec2,
    pub health: f32,
    pub max_health: f32,
}

// --- Aliens & Enemies ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlienType {
    Streuner, // Éclaireur léger (Map 1-1 débutant)
    Lordakia, // Chasseur agile en meute (Map 1-2)
    Sibelon,  // Titan dreadnought (Map PvP & Portails d'événements)
}

impl AlienType {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Streuner => "Streuner",
            Self::Lordakia => "Lordakia",
            Self::Sibelon => "Sibelon Dreadnought",
        }
    }

    pub fn max_health(&self) -> f32 {
        match self {
            Self::Streuner => 100.0,
            Self::Lordakia => 240.0,
            Self::Sibelon => 1600.0,
        }
    }

    pub fn max_shield(&self) -> f32 {
        match self {
            Self::Streuner => 40.0,
            Self::Lordakia => 120.0,
            Self::Sibelon => 900.0,
        }
    }

    pub fn speed(&self) -> f32 {
        match self {
            Self::Streuner => 170.0,
            Self::Lordakia => 240.0,
            Self::Sibelon => 125.0,
        }
    }

    pub fn laser_damage(&self) -> f32 {
        match self {
            Self::Streuner => 8.0,
            Self::Lordakia => 16.0,
            Self::Sibelon => 38.0,
        }
    }

    pub fn laser_cooldown(&self) -> f32 {
        match self {
            Self::Streuner => 0.8,
            Self::Lordakia => 0.45,
            Self::Sibelon => 0.35,
        }
    }

    pub fn aggro_range(&self) -> f32 {
        match self {
            Self::Streuner => 450.0,
            Self::Lordakia => 600.0,
            Self::Sibelon => 850.0,
        }
    }

    pub fn xp_reward(&self) -> u32 {
        match self {
            Self::Streuner => 35,
            Self::Lordakia => 90,
            Self::Sibelon => 550,
        }
    }

    pub fn credits_reward(&self) -> u32 {
        match self {
            Self::Streuner => 160,
            Self::Lordakia => 450,
            Self::Sibelon => 3200,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alien {
    pub id: u64,
    pub alien_type: AlienType,
    pub position: Vec2,
    pub velocity: Vec2,
    pub rotation: f32,
    pub health: f32,
    pub max_health: f32,
    pub shield: f32,
    pub max_shield: f32,
    pub target_player_id: Option<PlayerId>,
    pub last_shot_time: f32,
}

// --- Loot Box (Boîte de Fret) ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LootBox {
    pub id: u64,
    pub position: Vec2,
    pub credits: u32,
    pub mineral: Option<(MineralType, u32)>,
    pub scrap: u32,
    pub plasma_cores: u32,
    pub lifetime: f32,
}

// --- Talent Tree ---
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TalentTree {
    pub combat_laser_dmg: u32,       // 0..5 (+6% dmg par niveau)
    pub combat_fire_rate: u32,       // 0..5 (+5% cadence par niveau)
    pub defense_shield_max: u32,     // 0..5 (+10% shield max par niveau)
    pub defense_regen: u32,          // 0..5 (+15% regen par niveau)
    pub logistics_cargo: u32,        // 0..5 (+20% soute par niveau)
    pub logistics_mining_speed: u32, // 0..5 (+25% vitesse forage par niveau)
}

// --- Player Ship ---
pub type PlayerId = u64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerShip {
    pub id: PlayerId,
    pub username: String,
    pub ship_class: ShipClass,
    pub position: Vec2,
    pub velocity: Vec2,
    pub rotation: f32,
    pub health: f32,
    pub max_health: f32,
    pub shield: f32,
    pub max_shield: f32,
    pub credits: u32,
    pub score: u32,
    pub xp: u32,
    pub level: u32,
    pub next_level_xp: u32,
    pub talent_points: u32,
    pub talents: TalentTree,
    pub cargo: CargoHold,
    pub is_thrusting: bool,
    pub is_mining: bool,
    pub mining_target: Option<u64>,
    pub is_in_safe_zone: bool,
    pub is_alive: bool,
}

impl PlayerShip {
    pub fn new(id: PlayerId, username: String, class: ShipClass, position: Vec2) -> Self {
        let max_hp = class.base_health();
        let max_sh = class.base_shield();
        let cargo_cap = class.base_cargo();

        Self {
            id,
            username,
            ship_class: class,
            position,
            velocity: Vec2::ZERO,
            rotation: 0.0,
            health: max_hp,
            max_health: max_hp,
            shield: max_sh,
            max_shield: max_sh,
            credits: 1000,
            score: 0,
            xp: 0,
            level: 1,
            next_level_xp: 150,
            talent_points: 1,
            talents: TalentTree::default(),
            cargo: CargoHold::new(cargo_cap),
            is_thrusting: false,
            is_mining: false,
            mining_target: None,
            is_in_safe_zone: false,
            is_alive: true,
        }
    }

    pub fn add_xp(&mut self, amount: u32) -> bool {
        self.xp += amount;
        self.score += amount * 2;
        let mut leveled_up = false;
        while self.xp >= self.next_level_xp {
            self.xp -= self.next_level_xp;
            self.level += 1;
            self.talent_points += 1;
            self.next_level_xp = (self.next_level_xp as f32 * 1.45) as u32;
            leveled_up = true;
        }
        leveled_up
    }

    pub fn apply_talent_bonuses(&mut self) {
        let base_cargo = self.ship_class.base_cargo();
        let bonus_cargo_pct = self.talents.logistics_cargo as f32 * 0.20;
        self.cargo.max_capacity = (base_cargo as f32 * (1.0 + bonus_cargo_pct)) as u32;

        let base_sh = self.ship_class.base_shield();
        let bonus_sh_pct = self.talents.defense_shield_max as f32 * 0.10;
        self.max_shield = base_sh * (1.0 + bonus_sh_pct);
    }
}

// --- Lasers ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Laser {
    pub id: u64,
    pub shooter_id: u64, // PlayerId or Alien ID
    pub is_alien: bool,
    pub position: Vec2,
    pub velocity: Vec2,
    pub lifetime: f32,
    pub damage: f32,
    pub color_rgba: [f32; 4],
}

// --- Maps & Portals ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MapId {
    Map1_1,     // Secteur 1-1 • Base Principale (PvE Débutant)
    Map1_2,     // Secteur 1-2 • Ceinture d'Astéroïdes (PvE Intermédiaire)
    Map4_4,     // Secteur 4-4 • Zone de Conflit (PvP Total)
    EventArena, // Arène Événementielle (Mini-Event)
}

impl MapId {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Map1_1 => "Secteur 1-1 (QG PvE)",
            Self::Map1_2 => "Secteur 1-2 (Ceinture Minérale)",
            Self::Map4_4 => "Secteur 4-4 (Zone Contestée PvP)",
            Self::EventArena => "Faille Cosmique (Mini-Event)",
        }
    }

    pub fn is_pvp(&self) -> bool {
        matches!(self, Self::Map4_4)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventRiftType {
    RedBoss,      // Boss Titan Dreadnought
    GreenMining,  // Gisement pur de Terbium & Seprom
    PurpleSwarm,  // Vagues aliens d'invasion
    GoldTreasure, // Épave ancienne
}

impl EventRiftType {
    pub fn name(&self) -> &'static str {
        match self {
            Self::RedBoss => "🔴 Épreuve du Titan Sibelon",
            Self::GreenMining => "🟢 Filon Émeraude (Seprom/Terbium)",
            Self::PurpleSwarm => "🟣 Invasion Swarm (Survie)",
            Self::GoldTreasure => "🟡 Épave Antique (Loot Rare)",
        }
    }

    pub fn color_rgba(&self) -> [f32; 4] {
        match self {
            Self::RedBoss => [1.0, 0.15, 0.2, 1.0],
            Self::GreenMining => [0.1, 1.0, 0.35, 1.0],
            Self::PurpleSwarm => [0.85, 0.2, 1.0, 1.0],
            Self::GoldTreasure => [1.0, 0.85, 0.15, 1.0],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PortalType {
    MapJump { target_map: MapId, target_pos: Vec2 },
    EventRift { event_type: EventRiftType, time_left: f32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portal {
    pub id: u64,
    pub position: Vec2,
    pub radius: f32,
    pub portal_type: PortalType,
}

// --- Space Base ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceBase {
    pub position: Vec2,
    pub radius: f32,
    pub name: String,
}

// --- Crafting Recipes ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CraftRecipe {
    pub id: &'static str,
    pub name: &'static str,
    pub cost_prometium: u32,
    pub cost_endurium: u32,
    pub cost_terbium: u32,
    pub cost_scrap: u32,
    pub cost_credits: u32,
    pub description: &'static str,
}

pub const CRAFT_RECIPES: &[CraftRecipe] = &[
    CraftRecipe {
        id: "alloy_prometid",
        name: "Alliage Prometid",
        cost_prometium: 6,
        cost_endurium: 4,
        cost_terbium: 0,
        cost_scrap: 2,
        cost_credits: 150,
        description: "Alliage raffiné vendu au comptoir ou utilisé pour renforcer la coque (+350 C).",
    },
    CraftRecipe {
        id: "shield_booster",
        name: "Générateur Bouclier B0-2",
        cost_prometium: 10,
        cost_endurium: 8,
        cost_terbium: 5,
        cost_scrap: 5,
        cost_credits: 600,
        description: "Augmente de façon permanente la capacité de bouclier de +20 points.",
    },
    CraftRecipe {
        id: "laser_lf3",
        name: "Faisceau Laser LF-3",
        cost_prometium: 12,
        cost_endurium: 10,
        cost_terbium: 8,
        cost_scrap: 6,
        cost_credits: 1200,
        description: "Surcharge vos canons pour infliger +6 dégâts supplémentaires par tir.",
    },
    CraftRecipe {
        id: "cargo_extender",
        name: "Module Soute Compressée",
        cost_prometium: 15,
        cost_endurium: 12,
        cost_terbium: 4,
        cost_scrap: 8,
        cost_credits: 800,
        description: "Augmente la capacité maximale de la soute de +40 kg.",
    },
];

// --- World Snapshot ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSnapshot {
    pub tick: u64,
    pub server_time_ms: u64,
    pub current_map: MapId,
    pub players: Vec<PlayerShip>,
    pub aliens: Vec<Alien>,
    pub lasers: Vec<Laser>,
    pub minerals: Vec<Mineral>,
    pub loot_boxes: Vec<LootBox>,
    pub portals: Vec<Portal>,
}

// --- Network Messages ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    Auth { token: String },
    Input {
        thrust: bool,
        target_angle: f32,
        move_vec: Vec2,
    },
    Shoot,
    StartMining { mineral_id: u64 },
    StopMining,
    SelectClass { class: ShipClass },
    UpgradeTalent { talent_index: u32 },
    SellCargo,
    Craft { recipe_index: u32 },
    JumpPortal { portal_id: u64 },
    Respawn,
    Ping { client_time: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    AuthSuccess {
        player_id: PlayerId,
        username: String,
        ship: PlayerShip,
        current_map: MapId,
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
    Notification {
        title: String,
        message: String,
        color_rgba: [f32; 4],
    },
    Pong {
        client_time: u64,
        server_time: u64,
    },
}

// --- Bincode Serialization Helpers ---
pub fn serialize_packet<T: Serialize>(packet: &T) -> Result<Vec<u8>, String> {
    bincode::serde::encode_to_vec(packet, bincode::config::standard())
        .map_err(|e| format!("Serialization error: {}", e))
}

pub fn deserialize_packet<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, String> {
    bincode::serde::decode_from_slice(bytes, bincode::config::standard())
        .map(|(val, _)| val)
        .map_err(|e| format!("Deserialization error: {}", e))
}

// --- Inertial Physics Calculation ---
pub fn apply_ship_physics(ship: &mut PlayerShip, move_vec: Vec2, target_angle: f32, dt: f32) {
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

    let is_moving = move_vec.length_squared() > 0.001;
    ship.is_thrusting = is_moving;

    if is_moving {
        let thrust_dir = move_vec.normalize();
        let accel = ship.ship_class.base_acceleration();
        ship.velocity += thrust_dir * (accel * dt);

        let max_speed = ship.ship_class.base_speed();
        let speed = ship.velocity.length();
        if speed > max_speed {
            ship.velocity = ship.velocity.normalize() * max_speed;
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
