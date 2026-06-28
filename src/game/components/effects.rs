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

/// Intensity tier for hit effects based on damage percentage.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HitIntensity {
    Light,    // < 10% enemy HP
    Medium,   // 10-25% enemy HP
    Heavy,    // 25-50% enemy HP
    Critical, // > 50% enemy HP
}

impl HitIntensity {
    /// Calculate intensity based on damage as percentage of max HP.
    pub fn from_damage(damage: i32, max_hp: i32) -> Self {
        if max_hp <= 0 {
            return HitIntensity::Light;
        }
        let percent = (damage as f32 / max_hp as f32) * 100.0;
        if percent > 50.0 {
            HitIntensity::Critical
        } else if percent > 25.0 {
            HitIntensity::Heavy
        } else if percent > 10.0 {
            HitIntensity::Medium
        } else {
            HitIntensity::Light
        }
    }

    /// Hitstop duration in seconds.
    pub fn hitstop_duration(&self) -> f32 {
        match self {
            HitIntensity::Light => 0.02,
            HitIntensity::Medium => 0.04,
            HitIntensity::Heavy => 0.06,
            HitIntensity::Critical => 0.10,
        }
    }

    /// Screen shake trauma amount (0.0 to 1.0).
    pub fn shake_trauma(&self) -> f32 {
        match self {
            HitIntensity::Light => 0.1,
            HitIntensity::Medium => 0.2,
            HitIntensity::Heavy => 0.35,
            HitIntensity::Critical => 0.5,
        }
    }

    /// Number of impact particles to spawn.
    pub fn particle_count(&self) -> usize {
        match self {
            HitIntensity::Light => 4,
            HitIntensity::Medium => 8,
            HitIntensity::Heavy => 14,
            HitIntensity::Critical => 22,
        }
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

/// Short room-title text that fades after entering a room.
#[derive(Component)]
pub struct RoomBanner {
    pub life: Timer,
}

/// Remaining real-seconds of impact freeze.
#[derive(Resource, Default)]
pub struct HitStop(pub f32);

/// Runtime combat tuning overlay, toggled in-game.
#[derive(Resource, Default)]
pub struct CombatDebug {
    pub enabled: bool,
}

/// Ephemeral visual spawned for the combat debug overlay.
#[derive(Component)]
pub struct CombatDebugVisual;

/// Gentle vertical bob, used by floating pickups.
#[derive(Component)]
pub struct Bob {
    pub base_y: f32,
    pub amplitude: f32,
    pub speed: f32,
    pub phase: f32,
}

/// Low health vignette overlay that pulses when HP is low.
#[derive(Component)]
pub struct LowHealthVignette;

/// Fading afterimage trail left behind during dash.
#[derive(Component)]
pub struct Afterimage {
    pub life: Timer,
}
