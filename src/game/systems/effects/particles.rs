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
            Particle {
                velocity: dir * speed,
                life: Timer::from_seconds(0.34, TimerMode::Once),
            },
            RoomEntity,
        ));
    }
}

/// Impact particles scaled by hit intensity.
pub fn spawn_impact_particles(
    commands: &mut Commands,
    art: &GameArt,
    pos: Vec2,
    intensity: HitIntensity,
) {
    use std::f32::consts::TAU;

    let count = intensity.particle_count();
    let base_size = match intensity {
        HitIntensity::Light => 10.0,
        HitIntensity::Medium => 14.0,
        HitIntensity::Heavy => 18.0,
        HitIntensity::Critical => 24.0,
    };

    // Color shifts from white (light) to bright yellow/gold (critical)
    let color = match intensity {
        HitIntensity::Light => Color::srgb(1.0, 1.0, 1.0),
        HitIntensity::Medium => Color::srgb(1.0, 0.95, 0.8),
        HitIntensity::Heavy => Color::srgb(1.0, 0.85, 0.4),
        HitIntensity::Critical => Color::srgb(1.0, 0.7, 0.2),
    };

    for i in 0..count {
        let angle = i as f32 / count as f32 * TAU;
        let dir = Vec2::new(angle.cos(), angle.sin());
        let speed = 120.0 + (i % 5) as f32 * 30.0;
        let size = base_size + (i % 3) as f32 * 4.0;

        commands.spawn((
            art.image_sprite(&art.orb, Vec2::splat(size), color),
            Transform::from_translation(pos.extend(4.0)),
            Particle {
                velocity: dir * speed,
                life: Timer::from_seconds(0.28, TimerMode::Once),
            },
            RoomEntity,
        ));
    }
}

/// Dust particles spawned at player feet while moving.
pub fn spawn_dust_puff(commands: &mut Commands, art: &GameArt, pos: Vec2) {
    use std::f32::consts::TAU;

    // Spawn 2-3 small dust particles
    let count = 2 + (rand::random::<u32>() % 2) as usize;
    for i in 0..count {
        let angle = (i as f32 / count as f32 * TAU) + (rand::random::<f32>() - 0.5) * 0.5;
        let dir = Vec2::new(angle.cos(), angle.sin());
        let speed = 20.0 + rand::random::<f32>() * 30.0;
        let size = 5.0 + rand::random::<f32>() * 3.0;

        commands.spawn((
            art.image_sprite(
                &art.orb,
                Vec2::splat(size),
                Color::srgba(0.65, 0.60, 0.55, 0.7),
            ),
            Transform::from_translation(pos.extend(0.5)),
            Particle {
                velocity: dir * speed,
                life: Timer::from_seconds(0.25, TimerMode::Once),
            },
            RoomEntity,
        ));
    }
}

/// Combo milestone visual burst effect.
pub fn spawn_combo_burst(commands: &mut Commands, art: &GameArt, pos: Vec2, combo_count: u32) {
    use std::f32::consts::TAU;

    // Color-code by combo tier
    let color = match combo_count {
        5..=9 => Color::srgb(1.0, 0.9, 0.3),   // Gold for 5-9
        10..=14 => Color::srgb(1.0, 0.6, 0.2), // Orange for 10-14
        _ => Color::srgb(1.0, 0.3, 0.5),       // Pink/magenta for 15+
    };

    // Spawn expanding ring
    commands.spawn((
        art.image_sprite(&art.orb, Vec2::splat(100.0), color.with_alpha(0.8)),
        Transform::from_translation(pos.extend(4.0)).with_scale(Vec3::splat(0.1)),
        Explosion {
            life: Timer::from_seconds(0.35, TimerMode::Once),
            max_scale: 1.5,
        },
        RoomEntity,
    ));

    // Spawn 12 particles in a circle
    for i in 0..12 {
        let angle = i as f32 / 12.0 * TAU;
        let dir = Vec2::new(angle.cos(), angle.sin());
        commands.spawn((
            art.image_sprite(&art.orb, Vec2::splat(10.0), color),
            Transform::from_translation(pos.extend(4.5)),
            Particle {
                velocity: dir * 150.0,
                life: Timer::from_seconds(0.4, TimerMode::Once),
            },
            RoomEntity,
        ));
    }
}

/// Fire particles for Flaming Edge sword.
pub fn spawn_flame_particles(commands: &mut Commands, art: &GameArt, pos: Vec2, dir: Vec2) {
    for i in 0..5 {
        let spread = (rand::random::<f32>() - 0.5) * 0.8;
        let angle = dir.y.atan2(dir.x) + spread;
        let offset = Vec2::new(angle.cos(), angle.sin()) * (i as f32 * 0.2 * 20.0);
        let particle_pos = pos + offset;

        commands.spawn((
            art.image_sprite(
                &art.orb,
                Vec2::splat(12.0 + rand::random::<f32>() * 8.0),
                Color::srgb(1.0, 0.5 + rand::random::<f32>() * 0.3, 0.1),
            ),
            Transform::from_translation(particle_pos.extend(2.8)),
            Particle {
                velocity: dir * 40.0 + Vec2::new(rand::random::<f32>() * 20.0 - 10.0, 30.0),
                life: Timer::from_seconds(0.2, TimerMode::Once),
            },
            RoomEntity,
        ));
    }
}

/// Spark particles for Greatpurr sword heavy impacts.
pub fn spawn_spark_particles(commands: &mut Commands, art: &GameArt, pos: Vec2) {
    use std::f32::consts::TAU;

    for i in 0..8 {
        let angle = i as f32 / 8.0 * TAU + rand::random::<f32>() * 0.3;
        let dir = Vec2::new(angle.cos(), angle.sin());
        let speed = 80.0 + rand::random::<f32>() * 60.0;

        commands.spawn((
            art.image_sprite(
                &art.orb,
                Vec2::splat(6.0 + rand::random::<f32>() * 4.0),
                Color::srgb(1.0, 0.95, 0.7),
            ),
            Transform::from_translation(pos.extend(3.0)),
            Particle {
                velocity: dir * speed,
                life: Timer::from_seconds(0.18, TimerMode::Once),
            },
            RoomEntity,
        ));
    }
}

/// Leaf particles for Nine Lives sword.
pub fn spawn_leaf_particles(commands: &mut Commands, art: &GameArt, pos: Vec2, dir: Vec2) {
    for i in 0..6 {
        let spread = (rand::random::<f32>() - 0.5) * 1.2;
        let angle = dir.y.atan2(dir.x) + spread;

        commands.spawn((
            art.image_sprite(
                &art.orb,
                Vec2::splat(8.0 + rand::random::<f32>() * 6.0),
                Color::srgb(0.3 + rand::random::<f32>() * 0.2, 0.8, 0.35),
            ),
            Transform::from_translation(pos.extend(2.9)),
            Particle {
                velocity: Vec2::new(angle.cos(), angle.sin()) * 50.0
                    + Vec2::new(rand::random::<f32>() * 30.0 - 15.0, 20.0),
                life: Timer::from_seconds(0.35, TimerMode::Once),
            },
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
        PickupKind::Currency(_) => Color::srgb(1.0, 0.85, 0.0), // Gold color
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
            Particle {
                velocity: dir * speed,
                life: Timer::from_seconds(0.28, TimerMode::Once),
            },
            RoomEntity,
        ));
    }

    commands.spawn((
        art.image_sprite(&art.orb, Vec2::splat(40.0), color.with_alpha(0.6)),
        Transform::from_translation(pos.extend(3.0)),
        Particle {
            velocity: Vec2::ZERO,
            life: Timer::from_seconds(0.18, TimerMode::Once),
        },
        RoomEntity,
    ));
}
