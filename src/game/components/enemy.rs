use bevy::prelude::*;

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
#[derive(Clone, Copy, PartialEq, Eq)]
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