use super::persistent::{Modifier, PersistentState};
use crate::game::ability::{Ability, AbilitySlot};
use crate::game::sword::Sword;
use bevy::prelude::*;

/// Main game state for menu/pause/game over screens.
#[derive(Clone, Copy, PartialEq, Eq, Default, States, Debug, Hash)]
pub enum GameState {
    #[default]
    MainMenu,
    Playing,
    Paused,
    GameOver,
}

/// Which gameplay dimension is active. Only exists while `GameState::Playing`.
#[derive(SubStates, Clone, Copy, PartialEq, Eq, Default, Debug, Hash)]
#[source(GameState = GameState::Playing)]
pub enum GameMode {
    #[default]
    TwoD,
    Depth,
}

/// Room-level phase during gameplay.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Fighting,
    RoomCleared,
    // Note: GameOver moved to GameState
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RoomKind {
    Combat,
    Rest,
    Treasure,
    Boss,
    DepthTransition,
    DepthArena,
    DepthBoss,
    Shop,
}

impl RoomKind {
    pub fn is_depth(self) -> bool {
        matches!(self, Self::DepthTransition | Self::DepthArena | Self::DepthBoss)
    }
}

/// Decaying camera shake intensity.
#[derive(Resource, Default)]
pub struct ScreenShake {
    pub trauma: f32,
}

#[derive(Resource)]
pub struct RunState {
    pub floor: u32,
    pub room: u32,
    pub rooms_per_floor: u32,
    pub phase: Phase,
    pub player_hp: i32,
    pub player_max_hp: i32,
    pub damage_bonus: i32,
    pub player_speed: f32,
    pub projectile_speed: f32,
    pub swing_timer: Timer,
    pub sword: Sword,
    pub invuln: Timer,
    pub abilities: Vec<AbilitySlot>,
    pub room_seed: u64,
    pub current_room: RoomKind,
    pub combo_count: u32,
    pub combo_timer: Timer,
    pub best_combo: u32,
    pub active_modifiers: Vec<(Modifier, u32)>,
}

impl RunState {
    pub fn swing_damage(&self) -> i32 {
        let base = self.sword.damage + self.damage_bonus + self.combo_damage_bonus();
        if self.has_modifier(Modifier::GlassCannon) {
            base * 2
        } else {
            base
        }
    }

    pub fn combo_damage_bonus(&self) -> i32 {
        (self.combo_count / 5).min(3) as i32
    }

    pub fn has_modifier(&self, modifier: Modifier) -> bool {
        self.active_modifiers.iter().any(|(m, _)| *m == modifier)
    }

    pub fn effective_speed(&self) -> f32 {
        if self.has_modifier(Modifier::SpeedDemon) {
            self.player_speed * 1.5
        } else {
            self.player_speed
        }
    }

    pub fn equip(&mut self, sword: Sword) {
        self.sword = sword;
        self.swing_timer = Timer::from_seconds(sword.cooldown, TimerMode::Once);
        let dur = self.swing_timer.duration();
        self.swing_timer.tick(dur);
    }
}

impl RunState {
    pub fn new(persistent: &PersistentState) -> Self {
        let base_hp = 12 + persistent.starting_hp_bonus;
        let base_damage = persistent.starting_damage_bonus;
        let base_speed = 270.0 + persistent.starting_speed_bonus;

        let sword = Sword::starter();
        let mut swing_timer = Timer::from_seconds(sword.cooldown, TimerMode::Once);
        let dur = swing_timer.duration();
        swing_timer.tick(dur);
        let mut invuln = Timer::from_seconds(0.3, TimerMode::Once);
        let inv_dur = invuln.duration();
        invuln.tick(inv_dur);
        let combo_timer = Timer::from_seconds(2.0, TimerMode::Once);
        Self {
            floor: 1,
            room: 1,
            rooms_per_floor: 3,
            phase: Phase::Fighting,
            player_hp: base_hp,
            player_max_hp: base_hp,
            damage_bonus: base_damage,
            player_speed: base_speed,
            projectile_speed: 420.0,
            swing_timer,
            sword,
            invuln,
            abilities: vec![AbilitySlot::new(Ability::Dash)],
            room_seed: rand::random::<u64>(),
            current_room: RoomKind::Combat,
            combo_count: 0,
            combo_timer,
            best_combo: 0,
            active_modifiers: Vec::new(),
        }
    }
}

impl Default for RunState {
    fn default() -> Self {
        Self::new(&PersistentState::default())
    }
}
