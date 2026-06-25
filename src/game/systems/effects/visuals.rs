use crate::game::components::*;
use crate::game::state::ScreenShake;
use bevy::prelude::*;

pub fn tick_hit_flash(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Sprite, &BaseTint, &mut HitFlash)>,
) {
    for (entity, mut sprite, base, mut flash) in &mut query {
        flash.0.tick(time.delta());
        if flash.0.is_finished() {
            sprite.color = base.0;
            commands.entity(entity).remove::<HitFlash>();
        } else {
            sprite.color = Color::WHITE;
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