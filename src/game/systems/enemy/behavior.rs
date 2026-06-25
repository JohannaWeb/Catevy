use crate::game::components::EnemyKind;
use bevy::prelude::Vec2;

/// Wind-up time before an enemy commits its attack — the telegraph window.
pub fn windup_time(kind: EnemyKind) -> f32 {
    match kind {
        EnemyKind::Charger => 0.55,
        EnemyKind::Bomber => 0.4,
        EnemyKind::Boss => 0.45,
        _ => 0.3,
    }
}

/// How far away an enemy will commit to an attack.
pub fn trigger_range(kind: EnemyKind) -> f32 {
    match kind {
        EnemyKind::Charger => 320.0,
        EnemyKind::Bomber => 60.0,
        EnemyKind::Seeker => 340.0,
        EnemyKind::Caster => 320.0,
        EnemyKind::Boss => 460.0,
        EnemyKind::Summoner => 500.0,
        _ => 40.0,
    }
}

/// Cadence between telegraphed actions after a recovery.
pub fn action_cooldown(kind: EnemyKind) -> f32 {
    match kind {
        EnemyKind::Charger => 1.4,
        EnemyKind::Seeker => 1.1,
        EnemyKind::Caster => 1.3,
        EnemyKind::Boss => 0.8,
        EnemyKind::Summoner => 3.0,
        _ => 0.5,
    }
}

/// Per-kind movement during Chase: rush in, or kite to keep distance.
pub fn chase_velocity(kind: EnemyKind, dir: Vec2, distance: f32) -> Vec2 {
    match kind {
        EnemyKind::Seeker | EnemyKind::Caster | EnemyKind::Summoner => {
            let (near, far) = (220.0, 340.0);
            if distance < near { -dir }
            else if distance > far { dir }
            else { Vec2::new(-dir.y, dir.x) * 0.6 }
        }
        EnemyKind::Bruiser => dir * 0.7,
        _ => dir,
    }
}