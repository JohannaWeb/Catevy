mod player;
mod enemy;
mod projectiles;
mod effects;

// Re-export all public types
pub use player::*;
pub use enemy::*;
pub use projectiles::*;
pub use effects::*;

use bevy::prelude::*;
use std::time::Duration;

/// Exit door marker.
#[derive(Component)]
pub struct Door;

/// Marks entities that should be despawned when leaving a room.
#[derive(Component)]
pub struct RoomEntity;

/// HUD text marker.
#[derive(Component)]
pub struct HudText;

/// An obstacle that blocks movement and projectiles.
#[derive(Component)]
pub struct Obstacle {
    pub radius: f32,
    pub destructible: bool,
    pub hp: i32,
}

/// Drives sprite animation between idle/walk/attack clips.
#[derive(Component)]
pub struct CatAnimation {
    pub idle: (usize, usize),
    pub walk: (usize, usize),
    pub attack: Option<(usize, usize)>,
    pub moving: bool,
    pub attacking: bool,
    pub frame_timer: Timer,
}

impl CatAnimation {
    pub fn new(
        idle: (usize, usize),
        walk: (usize, usize),
        attack: Option<(usize, usize)>,
        fps: u8,
    ) -> Self {
        Self {
            idle,
            walk,
            attack,
            moving: false,
            attacking: false,
            frame_timer: Timer::new(Duration::from_secs_f32(1.0 / fps as f32), TimerMode::Repeating),
        }
    }

    pub fn clip(&self) -> (usize, usize) {
        if self.attacking {
            if let Some(attack) = self.attack {
                return attack;
            }
        }
        if self.moving { self.walk } else { self.idle }
    }

    pub fn start_attack(&mut self) {
        if self.attack.is_some() {
            self.attacking = true;
            self.frame_timer.reset();
        }
    }
}