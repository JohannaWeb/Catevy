use super::projectiles::PickupKind;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::hash::Hash;

/// Main enemy component with AI state and stats.
#[derive(Component)]
pub struct Enemy {
    pub kind: EnemyKind,
    pub hp: i32,
    pub max_hp: i32,
    pub damage: i32,
    pub speed: f32,
    pub elite: bool,
    pub splits: bool,
    pub explodes: bool,
    /// Generic counter: summons remaining, projectiles per volley, etc.
    pub ammo: i32,
    pub state: EnemyState,
    /// Counts down the current state; transitions when it finishes.
    pub state_timer: Timer,
    /// Gates how soon the next windup/charge/cast can begin.
    pub action_cd: Timer,
    pub charge_dir: Vec2,
    /// Whether a telegraph visual has been spawned for the current windup.
    pub has_telegraph: bool,
}

/// Behaviour phases shared by all enemies.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EnemyState {
    Chase,
    Windup,
    Charge,
    Recover,
}

/// All enemy types in the game.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EnemyKind {
    Hunter,
    Bruiser,
    Seeker,
    Boss,
    Charger,
    Bomber,
    Kitten,
    Splitter,
    Caster,
    Summoner,
    Scratcher,
    Chonker,
    ShadowCat,
    // New enemy types
    NecromancerCat, // Revives corpses as weaker enemies
    ShieldBearer,   // Frontal shield, must flank
    FlyingCat,      // Ignores obstacles
    MimicChest,     // Disguises as pickup, ambushes player
    Goliath,        // Grows when hit
}

/// Boss type determines special behaviors and phase transitions.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BossType {
    GoblinKing,
    Necromancer,
    Dragon,
}

/// Tracks boss phase for HP-based transitions.
#[derive(Component)]
pub struct BossPhases {
    pub boss_type: BossType,
    pub current_phase: u8,
}

/// Enemy synergy behavior - provides bonuses to nearby allies.
#[derive(Component)]
pub struct EnemySynergy {
    pub kind: SynergyKind,
    pub timer: Timer,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SynergyKind {
    Healer,
    Pack,
    Commander,
}

/// Bleed effect applied by Scratcher.
#[derive(Component)]
pub struct Bleed {
    pub damage: i32,
    pub tick_timer: Timer,
    pub remaining_ticks: u8,
}

/// NecromancerCat: Can revive nearby corpses as weaker enemies.
#[derive(Component)]
pub struct NecromancerCat {
    pub revive_range: f32,
    pub revive_timer: Timer,
    pub max_revives: i32,
}

/// ShieldBearer: Frontal shield blocks damage from front cone.
#[derive(Component)]
pub struct ShieldBearer {
    pub shield_angle: f32, // Cone angle (radians)
}

/// FlyingCat: Can fly over obstacles.
#[derive(Component)]
pub struct Flying;

/// MimicChest: Disguised as pickup until player approaches.
#[derive(Component)]
pub struct MimicChest {
    pub disguise_kind: PickupKind,
    pub triggered: bool,
    pub trigger_range: f32,
}

/// Goliath: Grows larger when damaged.
#[derive(Component)]
pub struct Goliath {
    pub growth_per_hit: f32,
    pub max_scale: f32,
}

/// Corpse: Marks position where enemy died, can be revived by NecromancerCat.
#[derive(Component)]
pub struct Corpse {
    pub kind: EnemyKind,
    pub floor: u32,
    pub was_elite: bool,
    pub despawn_timer: Timer,
}

/// Knockback: Visual squash/stretch animation during knockback.
#[derive(Component)]
pub struct Knockback {
    pub duration: Timer,
    pub direction: Vec2,
}
