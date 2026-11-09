// SPDX-License-Identifier: Apache-2.0
//! Persistent data (high scores, unlocked stages, settings) stored as RON in
//! the OS config directory: `$XDG_CONFIG_HOME/skypop` on Linux,
//! `%APPDATA%\QuietByte\SkyPop\config` on Windows.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::input::Bindings;

pub const MAX_SCORES: usize = 10;
const SAVE_VERSION: u32 = 1;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScoreEntry {
    pub name: String,
    pub score: u32,
    /// 1-based stage reached.
    pub stage: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaveData {
    pub version: u32,
    pub high_scores: Vec<ScoreEntry>,
    /// Highest stage (1-based) the player may start from.
    pub unlocked: u8,
    pub bindings: Bindings,
    pub fullscreen: bool,
    pub volume: f32,
}

impl Default for SaveData {
    fn default() -> Self {
        Self {
            version: SAVE_VERSION,
            high_scores: Vec::new(),
            unlocked: 1,
            bindings: Bindings::default(),
            fullscreen: false,
            volume: 0.8,
        }
    }
}

impl SaveData {
    fn path() -> Option<PathBuf> {
        directories::ProjectDirs::from("org", "QuietByte", "SkyPop")
            .map(|d| d.config_dir().join("save.ron"))
    }

    /// Loads the save file, falling back to defaults on any problem.
    pub fn load() -> Self {
        let Some(path) = Self::path() else {
            return Self::default();
        };
        let Ok(text) = std::fs::read_to_string(&path) else {
            return Self::default();
        };
        match ron::from_str::<SaveData>(&text) {
            Ok(mut s) => {
                if !s.bindings.is_valid() {
                    s.bindings = Bindings::default();
                }
                s.high_scores.truncate(MAX_SCORES);
                s.volume = s.volume.clamp(0.0, 1.0);
                s
            }
            Err(e) => {
                eprintln!(
                    "skypop: ignoring unreadable save file {}: {e}",
                    path.display()
                );
                Self::default()
            }
        }
    }

    /// Writes atomically (temp file + rename). Errors are reported, not fatal.
    pub fn store(&self) {
        let Some(path) = Self::path() else { return };
        let result = (|| -> std::io::Result<()> {
            if let Some(dir) = path.parent() {
                std::fs::create_dir_all(dir)?;
            }
            let text = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
                .map_err(std::io::Error::other)?;
            let tmp = path.with_extension("ron.tmp");
            std::fs::write(&tmp, text)?;
            std::fs::rename(&tmp, &path)
        })();
        if let Err(e) = result {
            eprintln!("skypop: could not save to {}: {e}", path.display());
        }
    }

    pub fn qualifies(&self, score: u32) -> bool {
        score > 0
            && (self.high_scores.len() < MAX_SCORES
                || self.high_scores.iter().any(|e| score > e.score))
    }

    /// Inserts a score in rank order; returns its 0-based rank if it made the table.
    pub fn add_score(&mut self, entry: ScoreEntry) -> Option<usize> {
        let rank = self
            .high_scores
            .iter()
            .position(|e| entry.score > e.score)
            .unwrap_or(self.high_scores.len());
        if rank >= MAX_SCORES {
            return None;
        }
        self.high_scores.insert(rank, entry);
        self.high_scores.truncate(MAX_SCORES);
        Some(rank)
    }

    pub fn unlock(&mut self, stage: u8) {
        self.unlocked = self.unlocked.max(stage);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scores_are_ranked_and_capped() {
        let mut s = SaveData::default();
        for i in 0..15u32 {
            s.add_score(ScoreEntry {
                name: "AAA".into(),
                score: i * 100,
                stage: 1,
            });
        }
        assert_eq!(s.high_scores.len(), MAX_SCORES);
        assert_eq!(s.high_scores[0].score, 1400);
        assert!(s.high_scores.windows(2).all(|w| w[0].score >= w[1].score));
        assert!(!s.qualifies(100));
        assert!(s.qualifies(5000));
    }

    #[test]
    fn save_round_trips() {
        let s = SaveData::default();
        let text = ron::to_string(&s).unwrap();
        let back: SaveData = ron::from_str(&text).unwrap();
        assert!(back.bindings.is_valid());
        assert_eq!(back.unlocked, 1);
    }
}
