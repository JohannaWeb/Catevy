use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use crate::game::ability::Ability;
use crate::game::components::EnemyKind;

/// Persistent state saved between runs.
#[derive(Resource, Clone, Serialize, Deserialize, Default)]
pub struct PersistentState {
    // Existing fields
    pub total_runs: u32,
    pub best_floor: u32,
    pub currency: u32,
    pub starting_hp_bonus: i32,
    pub starting_damage_bonus: i32,
    pub starting_speed_bonus: f32,
    pub unlocked_swords: Vec<usize>,

    // New: Unlockables
    pub unlocked_abilities: Vec<Ability>,
    pub unlocked_cosmetics: HashSet<CosmeticId>,

    // New: Achievements
    pub achievements: HashSet<AchievementId>,

    // New: Codex
    pub codex: HashMap<EnemyKind, CodexEntry>,

    // New: Statistics
    pub total_enemies_killed: u64,
    pub bosses_defeated: u32,
    pub secrets_found: u32,

    // New: Difficulty
    pub highest_difficulty_unlocked: u8,
    pub selected_difficulty: u8,
}

/// Codex entry for an enemy type.
#[derive(Clone, Serialize, Deserialize)]
pub struct CodexEntry {
    pub discovered: bool,
    pub times_killed: u32,
    pub lore_seen: bool,
}

impl Default for CodexEntry {
    fn default() -> Self {
        Self {
            discovered: false,
            times_killed: 0,
            lore_seen: false,
        }
    }
}

/// Cosmetic identifier for unlocks.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CosmeticId {
    // Cat skins
    CatOrangeTabby,
    CatBlack,
    CatWhite,
    CatSiamese,
    CatCalico,
    // Sword skins
    SwordFlame,
    SwordIce,
    SwordVoid,
    SwordRainbow,
}

/// Achievement identifier.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AchievementId {
    FirstBoss,
    Floor5,
    Floor10,
    Combo20,
    FlawlessRoom,
    Currency1000,
    Kills500,
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
