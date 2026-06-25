use crate::game::assets::GameArt;
use crate::game::components::*;
use bevy::prelude::*;

pub fn update_telegraphs(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Sprite, &mut Telegraph)>,
) {
    for (entity, mut sprite, mut telegraph) in &mut query {
        telegraph.life.tick(time.delta());
        if telegraph.life.is_finished() {
            commands.entity(entity).despawn();
            continue;
        }
        let frac = telegraph.life.fraction_remaining();
        sprite.color = sprite.color.with_alpha(frac * 0.35);
    }
}

pub fn spawn_telegraph(
    commands: &mut Commands,
    art: &GameArt,
    enemy_pos: Vec2,
    kind: EnemyKind,
    dir: Vec2,
    windup_duration: f32,
) {
    let (telegraph_kind, color) = match kind {
        EnemyKind::Charger => (TelegraphKind::Line { dir, length: 320.0 }, Color::srgb(1.0, 0.6, 0.3)),
        EnemyKind::Bomber => (TelegraphKind::Circle { radius: 120.0 }, Color::srgb(1.0, 0.4, 0.3)),
        EnemyKind::Boss => (TelegraphKind::Cone { dir, angle: 0.5, reach: 460.0 }, Color::srgb(0.9, 0.3, 0.35)),
        EnemyKind::Caster | EnemyKind::Seeker => (TelegraphKind::Cone { dir, angle: 0.3, reach: 320.0 }, Color::srgb(0.7, 0.5, 1.0)),
        _ => (TelegraphKind::Line { dir, length: 60.0 }, Color::srgb(1.0, 0.5, 0.4)),
    };

    let size = match &telegraph_kind {
        TelegraphKind::Line { length, .. } => Vec2::new(*length, 12.0),
        TelegraphKind::Circle { radius } => Vec2::splat(radius * 2.0),
        TelegraphKind::Cone { reach, .. } => Vec2::splat(*reach * 0.6),
    };

    let rotation = match &telegraph_kind {
        TelegraphKind::Line { dir, .. } | TelegraphKind::Cone { dir, .. } => Quat::from_rotation_z(dir.y.atan2(dir.x)),
        TelegraphKind::Circle { .. } => Quat::IDENTITY,
    };

    commands.spawn((
        art.image_sprite(&art.orb, size, color.with_alpha(0.35)),
        Transform { translation: enemy_pos.extend(0.5), rotation, ..default() },
        Telegraph { kind: telegraph_kind, life: Timer::from_seconds(windup_duration, TimerMode::Once) },
        RoomEntity,
    ));
}