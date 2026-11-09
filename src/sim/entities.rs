// SPDX-License-Identifier: Apache-2.0
//! Plain-data simulation entities. Everything here is `Clone + Serialize` so a
//! whole `World` can be snapshotted (rendering interpolation, LAN co-op).

use serde::{Deserialize, Serialize};

/// Base gravity for orbs, in logical px/s².
pub const GRAVITY: f32 = 1100.0;
/// Upward hop given to the two children of a split orb (px of rise).
pub const SPLIT_HOP: f32 = 80.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tier {
    XL,
    L,
    M,
    S,
}

impl Tier {
    pub fn radius(self) -> f32 {
        match self {
            Tier::XL => 46.0,
            Tier::L => 31.0,
            Tier::M => 19.0,
            Tier::S => 10.0,
        }
    }

    /// Height (px) an orb rises above the surface it bounces on. Fixed per
    /// tier, so every bounce of a given tier peaks at the same height.
    pub fn bounce_height(self) -> f32 {
        match self {
            Tier::XL => 450.0,
            Tier::L => 360.0,
            Tier::M => 270.0,
            Tier::S => 180.0,
        }
    }

    /// Launch speed that reaches exactly `bounce_height` under `GRAVITY`.
    pub fn bounce_speed(self) -> f32 {
        (2.0 * GRAVITY * self.bounce_height()).sqrt()
    }

    pub fn h_speed(self) -> f32 {
        match self {
            Tier::XL => 115.0,
            Tier::L => 128.0,
            Tier::M => 140.0,
            Tier::S => 152.0,
        }
    }

    pub fn score(self) -> u32 {
        match self {
            Tier::XL => 50,
            Tier::L => 100,
            Tier::M => 150,
            Tier::S => 200,
        }
    }

    pub fn next(self) -> Option<Tier> {
        match self {
            Tier::XL => Some(Tier::L),
            Tier::L => Some(Tier::M),
            Tier::M => Some(Tier::S),
            Tier::S => None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Orb {
    pub id: u32,
    pub tier: Tier,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub color: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Weapon {
    Harpoon,
    AnchorWire,
    DoubleWire,
}

impl Weapon {
    pub fn max_wires(self) -> usize {
        match self {
            Weapon::DoubleWire => 2,
            _ => 1,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Weapon::Harpoon => "HARPOON",
            Weapon::AnchorWire => "ANCHOR",
            Weapon::DoubleWire => "DOUBLE",
        }
    }
}

pub const WIRE_SPEED: f32 = 960.0;
pub const ANCHOR_STICK_TIME: f32 = 2.6;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Wire {
    pub id: u32,
    pub owner: u8,
    pub anchor: bool,
    pub x: f32,
    pub tip_y: f32,
    /// Remaining stick time once an anchor wire has latched on.
    pub stuck: Option<f32>,
}

pub const PLAYER_W: f32 = 44.0;
pub const PLAYER_H: f32 = 72.0;
pub const PLAYER_SPEED: f32 = 330.0;
pub const START_LIVES: u8 = 3;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Player {
    pub index: u8,
    /// Horizontal centre.
    pub x: f32,
    pub facing: i8,
    pub moving: bool,
    pub lives: u8,
    pub alive: bool,
    pub invuln: f32,
    pub shield: bool,
    pub weapon: Weapon,
    pub prev_fire: bool,
    /// Seconds since the last shot (drives the throw animation).
    pub shot_t: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlatformKind {
    Solid,
    Breakable,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Platform {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub kind: PlatformKind,
}

impl Platform {
    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }

    pub fn spans_x(&self, x: f32) -> bool {
        x >= self.x && x <= self.x + self.w
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DropKind {
    Shield,
    Freeze,
    Slow,
    Bonus,
    Anchor,
    Double,
}

pub const DROP_SIZE: f32 = 30.0;
pub const DROP_FALL_SPEED: f32 = 260.0;
pub const DROP_LIFETIME: f32 = 7.0;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Drop {
    pub id: u32,
    pub kind: DropKind,
    pub x: f32,
    pub y: f32,
    pub landed: bool,
    pub life: f32,
}

pub const BOSS_W: f32 = 200.0;
pub const BOSS_H: f32 = 96.0;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Boss {
    /// Centre of the hover-pod.
    pub x: f32,
    pub y: f32,
    pub hp: u16,
    pub max_hp: u16,
    pub t: f32,
    pub spawn_timer: f32,
    pub spawn_index: usize,
    pub flash: f32,
}

impl Boss {
    pub fn bottom(&self) -> f32 {
        self.y + BOSS_H * 0.5
    }

    pub fn spans_x(&self, x: f32) -> bool {
        (x - self.x).abs() <= BOSS_W * 0.5
    }

    /// 0 = full health, 1 = below half, 2 = below a quarter.
    pub fn phase(&self) -> u8 {
        let f = self.hp as f32 / self.max_hp as f32;
        if f <= 0.25 {
            2
        } else if f <= 0.5 {
            1
        } else {
            0
        }
    }
}
