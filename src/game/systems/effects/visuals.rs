use crate::game::components::*;
use crate::game::state::{RunState, ScreenShake};
use bevy::prelude::*;

pub fn tick_hit_flash(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Sprite, &BaseTint, &mut HitFlash, Option<&Player>)>,
) {
    for (entity, mut sprite, base, mut flash, player) in &mut query {
        flash.0.tick(time.delta());
        if flash.0.is_finished() {
            sprite.color = base.0;
            commands.entity(entity).remove::<HitFlash>();
        } else {
            sprite.color = if player.is_some() {
                Color::srgb(1.0, 0.28, 0.24)
            } else {
                Color::WHITE
            };
        }
    }
}

pub fn screen_shake(
    time: Res<Time>,
    mut shake: ResMut<ScreenShake>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
) {
    let Ok(mut camera) = camera_query.single_mut() else { return; };
    shake.trauma = (shake.trauma - time.delta_secs() * 1.8).max(0.0);
    let amount = shake.trauma * shake.trauma;
    if amount <= 0.0 {
        camera.translation.x = 0.0;
        camera.translation.y = 0.0;
        return;
    }
    let magnitude = 18.0 * amount;
    let t = time.elapsed_secs() * 58.0;
    camera.translation.x = (t * 1.31).sin() * magnitude;
    camera.translation.y = (t * 1.73).cos() * magnitude;
}

pub fn animate_cats(time: Res<Time>, mut query: Query<(&mut CatAnimation, &mut Sprite)>) {
    for (mut animation, mut sprite) in &mut query {
        animation.frame_timer.tick(time.delta());
        let (first, last) = animation.clip();
        let Some(atlas) = sprite.texture_atlas.as_mut() else { continue; };
        if atlas.index < first || atlas.index > last {
            atlas.index = first;
        } else if animation.frame_timer.just_finished() {
            if atlas.index >= last {
                if animation.attacking {
                    animation.attacking = false;
                    atlas.index = animation.clip().0;
                } else {
                    atlas.index = first;
                }
            } else {
                atlas.index += 1;
            }
        }
    }
}

pub fn update_room_banners(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut TextColor, &mut RoomBanner)>,
) {
    for (entity, mut transform, mut color, mut banner) in &mut query {
        banner.life.tick(time.delta());
        if banner.life.is_finished() {
            commands.entity(entity).despawn();
            continue;
        }
        transform.translation.y += 10.0 * time.delta_secs();
        color.0 = color.0.with_alpha(banner.life.fraction_remaining());
    }
}

pub fn update_low_health_warning(
    time: Res<Time>,
    run: Res<RunState>,
    mut query: Query<&mut ImageNode, With<LowHealthVignette>>,
) {
    let Ok(mut vignette) = query.single_mut() else { return; };

    let hp_percent = run.player_hp as f32 / run.player_max_hp as f32;

    // Only show warning below 30% HP
    if hp_percent > 0.30 {
        vignette.color = vignette.color.with_alpha(0.0);
        return;
    }

    // Pulse faster as HP gets lower
    // Base pulse speed: 3.0, increases up to 18.0 as HP approaches 0
    let pulse_speed = 3.0 + (0.30 - hp_percent) * 50.0;
    let pulse = (time.elapsed_secs() * pulse_speed).sin() * 0.5 + 0.5;

    // Base alpha increases as HP gets lower
    let base_alpha = (0.30 - hp_percent) / 0.30 * 0.5;
    vignette.color = vignette.color.with_alpha(base_alpha + pulse * 0.2);
}

pub fn update_afterimages(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Sprite, &mut Afterimage)>,
) {
    for (entity, mut sprite, mut afterimage) in &mut query {
        afterimage.life.tick(time.delta());
        if afterimage.life.is_finished() {
            commands.entity(entity).despawn();
        } else {
            let alpha = afterimage.life.fraction_remaining() * 0.6;
            sprite.color = sprite.color.with_alpha(alpha);
        }
    }
}

pub fn update_knockback(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut Knockback)>,
) {
    for (entity, mut transform, mut knockback) in &mut query {
        knockback.duration.tick(time.delta());
        if knockback.duration.is_finished() {
            commands.entity(entity).remove::<Knockback>();
            transform.scale = Vec3::splat(1.0);
        } else {
            // Squash in direction of knockback, stretch perpendicular
            let t = knockback.duration.fraction();
            let squash = 1.0 + (1.0 - t) * 0.2;
            let stretch = 1.0 - (1.0 - t) * 0.15;

            // Determine squash axis based on knockback direction
            if knockback.direction.x.abs() > knockback.direction.y.abs() {
                // Horizontal knockback: squash horizontally, stretch vertically
                transform.scale = Vec3::new(squash, stretch, 1.0);
            } else {
                // Vertical knockback: squash vertically, stretch horizontally
                transform.scale = Vec3::new(stretch, squash, 1.0);
            }
        }
    }
}
