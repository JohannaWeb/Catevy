use crate::game::components::*;
use bevy::prelude::*;
use bevy::time::{Real, Virtual};

/// Freezes virtual time while a hit-stop is active, counting down in real time.
pub fn apply_hitstop(
    real: Res<Time<Real>>,
    mut hitstop: ResMut<HitStop>,
    mut virtual_time: ResMut<Time<Virtual>>,
) {
    if hitstop.0 > 0.0 {
        hitstop.0 -= real.delta_secs();
        virtual_time.set_relative_speed(0.0);
    } else {
        virtual_time.set_relative_speed(1.0);
    }
}

pub fn spawn_damage_number(commands: &mut Commands, font: &Handle<Font>, pos: Vec2, amount: i32) {
    let drift = ((pos.x as i32 % 7) - 3) as f32 * 9.0;
    commands.spawn((
        Text2d::new(format!("{amount}")),
        TextFont { font: font.clone().into(), font_size: FontSize::Px(22.0), ..default() },
        TextColor(Color::srgb(1.0, 0.95, 0.55)),
        Transform::from_translation(pos.extend(6.0)),
        DamageNumber { life: Timer::from_seconds(0.55, TimerMode::Once), velocity: Vec2::new(drift, 72.0) },
        RoomEntity,
    ));
}

pub fn update_damage_numbers(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut TextColor, &mut DamageNumber)>,
) {
    let dt = time.delta_secs();
    for (entity, mut transform, mut color, mut number) in &mut query {
        number.life.tick(time.delta());
        if number.life.is_finished() {
            commands.entity(entity).despawn();
            continue;
        }
        transform.translation.x += number.velocity.x * dt;
        transform.translation.y += number.velocity.y * dt;
        color.0 = color.0.with_alpha(number.life.fraction_remaining());
    }
}