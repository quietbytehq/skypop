// SPDX-License-Identifier: Apache-2.0
//! Level definitions. Levels live in `data/levels/*.ron`; they are embedded
//! into the binary at build time and can be overridden at runtime by pointing
//! `SKYPOP_DATA_DIR` at a directory containing `levels/levelNN.ron` files.

use serde::{Deserialize, Serialize};

use super::entities::{DropKind, PlatformKind, Tier, Weapon};
use super::{ARENA_BOTTOM, ARENA_LEFT, ARENA_RIGHT, ARENA_TOP};

pub const LEVEL_COUNT: usize = 10;

const EMBEDDED: [&str; LEVEL_COUNT] = [
    include_str!("../../data/levels/level01.ron"),
    include_str!("../../data/levels/level02.ron"),
    include_str!("../../data/levels/level03.ron"),
    include_str!("../../data/levels/level04.ron"),
    include_str!("../../data/levels/level05.ron"),
    include_str!("../../data/levels/level06.ron"),
    include_str!("../../data/levels/level07.ron"),
    include_str!("../../data/levels/level08.ron"),
    include_str!("../../data/levels/level09.ron"),
    include_str!("../../data/levels/level10.ron"),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Decor {
    Clouds,
    Hills,
    City,
    Stars,
    Fortress,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Theme {
    pub sky_top: String,
    pub sky_bottom: String,
    pub accent: String,
    pub decor: Decor,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrbSpawn {
    pub tier: Tier,
    pub x: f32,
    #[serde(default = "default_orb_y")]
    pub y: f32,
    /// -1 = moving left, 1 = moving right.
    pub dir: i8,
    #[serde(default)]
    pub color: u8,
}

fn default_orb_y() -> f32 {
    160.0
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlatformDef {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub kind: PlatformKind,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BossDef {
    pub hp: u16,
    /// Seconds between orb drops at full health.
    pub spawn_interval: f32,
    /// Tiers the boss cycles through when dropping orbs.
    pub spawn_tiers: Vec<Tier>,
    /// Horizontal sweep speed multiplier.
    pub speed: f32,
    /// The boss stops dropping orbs while this many are on screen.
    pub max_orbs: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LevelDef {
    pub name: String,
    /// Seconds before TIME UP.
    pub time_limit: f32,
    /// Orb time-scale multiplier (bounce heights are unaffected).
    pub speed: f32,
    pub weapon: Weapon,
    pub drop_chance: f32,
    pub drop_table: Vec<DropKind>,
    pub theme: Theme,
    pub orbs: Vec<OrbSpawn>,
    #[serde(default)]
    pub platforms: Vec<PlatformDef>,
    #[serde(default)]
    pub boss: Option<BossDef>,
}

impl LevelDef {
    pub fn parse(src: &str) -> Result<Self, String> {
        let def: LevelDef = ron::from_str(src).map_err(|e| e.to_string())?;
        def.validate()?;
        Ok(def)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.time_limit <= 0.0 || self.speed <= 0.0 {
            return Err(format!(
                "{}: time_limit and speed must be positive",
                self.name
            ));
        }
        if !(0.0..=1.0).contains(&self.drop_chance) {
            return Err(format!("{}: drop_chance must be in 0..=1", self.name));
        }
        if self.drop_chance > 0.0 && self.drop_table.is_empty() {
            return Err(format!("{}: drop_table is empty", self.name));
        }
        if self.orbs.is_empty() && self.boss.is_none() {
            return Err(format!("{}: a level needs orbs or a boss", self.name));
        }
        for o in &self.orbs {
            let r = o.tier.radius();
            if o.x - r < ARENA_LEFT || o.x + r > ARENA_RIGHT || o.y - r < ARENA_TOP {
                return Err(format!(
                    "{}: orb at x={} is outside the arena",
                    self.name, o.x
                ));
            }
            if o.dir != -1 && o.dir != 1 {
                return Err(format!("{}: orb dir must be -1 or 1", self.name));
            }
        }
        for p in &self.platforms {
            if p.x < ARENA_LEFT || p.x + p.w > ARENA_RIGHT || p.y < ARENA_TOP {
                return Err(format!("{}: platform outside the arena", self.name));
            }
            // Platforms float above the players' heads.
            if p.y + p.h > ARENA_BOTTOM - 110.0 {
                return Err(format!("{}: platform too low (blocks players)", self.name));
            }
        }
        if let Some(b) = &self.boss {
            if b.hp == 0 || b.spawn_tiers.is_empty() || b.spawn_interval <= 0.0 {
                return Err(format!("{}: invalid boss definition", self.name));
            }
        }
        Ok(())
    }
}

/// Loads all levels, preferring files under `$SKYPOP_DATA_DIR/levels/`.
pub fn load_all() -> Result<Vec<LevelDef>, String> {
    let override_dir = std::env::var_os("SKYPOP_DATA_DIR").map(std::path::PathBuf::from);
    (0..LEVEL_COUNT)
        .map(|i| {
            let file = format!("level{:02}.ron", i + 1);
            let from_disk = override_dir
                .as_ref()
                .and_then(|d| std::fs::read_to_string(d.join("levels").join(&file)).ok());
            let src = from_disk.as_deref().unwrap_or(EMBEDDED[i]);
            LevelDef::parse(src).map_err(|e| format!("{file}: {e}"))
        })
        .collect()
}
