use crate::game::assets::GameArt;
use crate::game::components::*;
use crate::game::state::{Phase, RunState};
use crate::game::systems::effects::spawn_dust_puff;
use bevy::prelude::*;

use super::combat::{ARENA_HALF_HEIGHT, ARENA_HALF_WIDTH};

/// Track when the player last aimed with the mouse (to avoid fighting with movement)
#[derive(Resource, Default)]
pub struct LastMouseAim {
    pub direction: Option<Vec2>,
    pub timer: Timer,
}

/// Timer for spawning dust particles while moving
#[derive(Resource, Default)]
pub struct DustTimer(pub Timer);

pub fn player_movement(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    run: Res<RunState>,
    art: Res<GameArt>,
    mut last_aim: ResMut<LastMouseAim>,
    mut dust_timer: Local<DustTimer>,
    obstacles: Query<(&Transform, &Obstacle), Without<Player>>,
    mut query: Query<(&mut Transform, &mut Facing, &mut Sprite, &mut CatAnimation), With<Player>>,
    mut commands: Commands,
) {
    let Ok((mut transform, mut facing, mut sprite, mut anim)) = query.single_mut() else {
        return;
    };

    // Initialize dust timer on first run
    if dust_timer.0.duration().as_secs_f32() == 0.0 {
        dust_timer.0 = Timer::from_seconds(0.08, TimerMode::Once);
    }

    // Tick down the mouse aim timer
    last_aim.timer.tick(time.delta());
    if last_aim.timer.is_finished() {
        last_aim.direction = None;
    }

    if run.current_room.is_depth() {
        anim.moving = false;
        return;
    }

    if run.phase == Phase::GameOver {
        anim.moving = false;
        return;
    }

    let mut direction = Vec2::ZERO;
    if keyboard.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }

    if direction.length_squared() == 0.0 {
        anim.moving = false;
        return;
    }

    anim.moving = true;
    let dir = direction.normalize();
    let velocity = dir * run.effective_speed() * time.delta_secs();

    let player_radius = 18.0;
    let current_pos = transform.translation.truncate();

    let mut new_x =
        (transform.translation.x + velocity.x).clamp(-ARENA_HALF_WIDTH, ARENA_HALF_WIDTH);
    for (obs_transform, obstacle) in &obstacles {
        let obs_pos = obs_transform.translation.truncate();
        let min_dist = player_radius + obstacle.radius;
        let dy = (current_pos.y - obs_pos.y).abs();
        if dy < min_dist {
            let dx = new_x - obs_pos.x;
            if dx.abs() < min_dist {
                new_x = obs_pos.x + min_dist.copysign(dx);
            }
        }
    }

    let mut new_y =
        (transform.translation.y + velocity.y).clamp(-ARENA_HALF_HEIGHT, ARENA_HALF_HEIGHT);
    for (obs_transform, obstacle) in &obstacles {
        let obs_pos = obs_transform.translation.truncate();
        let min_dist = player_radius + obstacle.radius;
        let dx = (new_x - obs_pos.x).abs();
        if dx < min_dist {
            let dy = new_y - obs_pos.y;
            if dy.abs() < min_dist {
                new_y = obs_pos.y + min_dist.copysign(dy);
            }
        }
    }

    transform.translation.x = new_x;
    transform.translation.y = new_y;

    // Spawn dust particles while moving
    dust_timer.0.tick(time.delta());
    if dust_timer.0.is_finished() {
        let dust_pos = current_pos + Vec2::new(0.0, -15.0);
        spawn_dust_puff(&mut commands, &art, dust_pos);
        dust_timer.0.reset();
    }

    // Only update facing from movement if we haven't aimed with the mouse recently
    if last_aim.direction.is_none() {
        facing.0 = dir;
        if dir.x.abs() > 0.05 {
            sprite.flip_x = dir.x < 0.0;
        }
    }
}

pub fn dash_flicker(run: Res<RunState>, mut query: Query<&mut Sprite, With<Player>>) {
    let Ok(mut sprite) = query.single_mut() else {
        return;
    };
    if run.invuln.is_finished() {
        if sprite.color.alpha() < 1.0 {
            sprite.color = sprite.color.with_alpha(1.0);
        }
    } else {
        let blink = ((run.invuln.elapsed_secs() * 22.0) as i32) % 2 == 0;
        sprite.color = sprite.color.with_alpha(if blink { 0.4 } else { 0.9 });
    }
}
