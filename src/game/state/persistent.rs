use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Persistent state saved between runs.
#[derive(Resource, Clone, Serialize, Deserialize, Default)]
pub struct PersistentState {
    pub total_runs: u32,
    pub best_floor: u32,
    pub currency: u32,
    pub starting_hp_bonus: i32,
    pub starting_damage_bonus: i32,
    pub starting_speed_bonus: f32,
    pub unlocked_swords: Vec<usize>,
}

impl PersistentState {
    pub fn save_path() -> PathBuf {
        let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("catevy");
        fs::create_dir_all(&path).ok();
        path.push("save.json");
        path
    }

    pub fn load() -> Self {
        fs::read_to_string(Self::save_path())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            fs::write(Self::save_path(), json).ok();
        }
    }
}

/// Risk/reward modifiers that affect gameplay.
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Modifier {
    GlassCannon,
    SpeedDemon,
    Vampire,
    TreasureHunter,
}