use bevy::prelude::*;

/// Visual telegraph showing where an enemy is about to attack.
#[derive(Component)]
pub struct Telegraph {
    pub kind: TelegraphKind,
    pub life: Timer,
}

pub enum TelegraphKind {
    Line { dir: Vec2, length: f32 },
    Circle { radius: f32 },
    Cone { dir: Vec2, angle: f32, reach: f32 },
}

/// A short-lived particle that fades and shrinks.
#[derive(Component)]
pub struct Particle {
    pub velocity: Vec2,
    pub life: Timer,
}

/// Brief white flash when an entity takes damage.
#[derive(Component)]
pub struct HitFlash(pub Timer);

impl HitFlash {
    pub fn new() -> Self {
        Self(Timer::from_seconds(0.10, TimerMode::Once))
    }
}

/// The resting tint of a sprite, restored after a hit flash.
#[derive(Component)]
pub struct BaseTint(pub Color);

/// Floating combat text that drifts up and fades.
#[derive(Component)]
pub struct DamageNumber {
    pub life: Timer,
    pub velocity: Vec2,
}

/// Remaining real-seconds of impact freeze.
#[derive(Resource, Default)]
pub struct HitStop(pub f32);

/// Gentle vertical bob, used by floating pickups.
#[derive(Component)]
pub struct Bob {
    pub base_y: f32,
    pub amplitude: f32,
    pub speed: f32,
    pub phase: f32,
}