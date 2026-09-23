use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize,
Deserialize)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn normalize(self) -> Self {
        let len = self.length();
        if len == 0.0 {
            return Self::zero();
        }
        Self { x: self.x / len, y: self.y / len }
    }
}

#[derive(Debug, Clone, Copy, PartialEq,
Eq, Serialize, Deserialize)]
pub enum Input {
    Up,
    Down,
    Left,
    Right,
    Fire,
}

#[derive(Debug, Clone, Serialize,
Deserialize)]
pub struct PlayerState {
    pub id: u32,
    pub position: Vec2,
    pub velocity: Vec2,
    pub rotation: f32,
}

#[derive(Debug, Clone, Serialize,
Deserialize)]
pub enum NetMessage {
    Input { player_id: u32, input: Input },
    State { player: PlayerState },
    FullState { players: Vec<PlayerState> },
}
