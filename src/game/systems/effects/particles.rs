use crate::game::assets::GameArt;
use crate::game::components::*;
use bevy::prelude::*;

pub fn update_particles(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut Sprite, &mut Particle)>,
) {
    let delta = time.delta_secs();
    for (entity, mut transform, mut sprite, mut particle) in &mut query {
        particle.life.tick(time.delta());
        if particle.life.is_finished() {
            commands.entity(entity).despawn();
            continue;
        }
        transform.translation.x += particle.velocity.x * delta;
        transform.translation.y += particle.velocity.y * delta;
        let frac = particle.life.fraction_remaining();
        transform.scale = Vec3::splat(frac.max(0.1));
        sprite.color = sprite.color.with_alpha(frac);
    }
}

/// A small burst of dust particles, e.g. when an enemy dies or a pickup is grabbed.
pub fn spawn_poof(commands: &mut Commands, art: &GameArt, pos: Vec2) {
    use std::f32::consts::TAU;
    for i in 0..8 {
        let angle = i as f32 / 8.0 * TAU;
        let dir = Vec2::new(angle.cos(), angle.sin());
        let speed = 110.0 + (i % 3) as f32 * 35.0;
        commands.spawn((
            art.image_sprite(&art.orb, Vec2::splat(16.0), Color::srgb(0.95, 0.95, 1.0)),
            Transform::from_translation(pos.extend(3.5)),
            Particle { velocity: dir * speed, life: Timer::from_seconds(0.34, TimerMode::Once) },
            RoomEntity,
        ));
    }
}

/// A colorful burst effect when collecting a pickup, with type-specific colors.
pub fn spawn_pickup_pop(commands: &mut Commands, art: &GameArt, pos: Vec2, kind: PickupKind) {
    use std::f32::consts::TAU;

    let color = match kind {
        PickupKind::Heal(_) => Color::srgb(0.4, 1.0, 0.5),
        PickupKind::DamageUp(_) => Color::srgb(1.0, 0.5, 0.4),
        PickupKind::SpeedUp(_) => Color::srgb(0.5, 0.8, 1.0),
        PickupKind::MaxHpUp(_) => Color::srgb(1.0, 0.9, 0.4),
        PickupKind::SwordDrop(_) => Color::srgb(1.0, 0.85, 0.5),
        PickupKind::AbilityDrop(_) => Color::srgb(0.8, 0.6, 1.0),
    };

    let particle_count = match kind {
        PickupKind::SwordDrop(_) | PickupKind::AbilityDrop(_) => 16,
        _ => 12,
    };

    for i in 0..particle_count {
        let angle = i as f32 / particle_count as f32 * TAU;
        let dir = Vec2::new(angle.cos(), angle.sin());
        let speed = 80.0 + (i % 4) as f32 * 25.0;

        commands.spawn((
            art.image_sprite(&art.orb, Vec2::splat(14.0), color),
            Transform::from_translation(pos.extend(3.5)),
            Particle { velocity: dir * speed, life: Timer::from_seconds(0.28, TimerMode::Once) },
            RoomEntity,
        ));
    }

    commands.spawn((
        art.image_sprite(&art.orb, Vec2::splat(40.0), color.with_alpha(0.6)),
        Transform::from_translation(pos.extend(3.0)),
        Particle { velocity: Vec2::ZERO, life: Timer::from_seconds(0.18, TimerMode::Once) },
        RoomEntity,
    ));
}