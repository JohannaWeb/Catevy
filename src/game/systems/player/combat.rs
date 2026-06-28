use crate::game::assets::{GameArt, KNIGHT_ATTACK, Sfx};
use crate::game::components::*;
use crate::game::spawning::spawn_projectile;
use crate::game::state::{Phase, RunState};
use bevy::audio::{AudioSource, PlaybackSettings, Volume};
use bevy::prelude::*;
use std::collections::HashSet;

use super::movement::LastMouseAim;

pub const ARENA_HALF_WIDTH: f32 = 540.0;
pub const ARENA_HALF_HEIGHT: f32 = 300.0;
pub const DOOR_DISTANCE: f32 = 64.0;

/// Plays a one-shot sound that despawns itself when finished.
pub fn play(commands: &mut Commands, clip: &Handle<AudioSource>, volume: f32) {
    commands.spawn((
        AudioPlayer::new(clip.clone()),
        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(volume)),
    ));
}

pub fn player_swing(
    time: Res<Time>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut run: ResMut<RunState>,
    mut last_aim: ResMut<LastMouseAim>,
    art: Res<GameArt>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    sfx: Res<Sfx>,
    mut player_query: Query<
        (&Transform, &mut Facing, &mut Sprite, &mut CatAnimation),
        With<Player>,
    >,
    mut commands: Commands,
) {
    run.swing_timer.tick(time.delta());
    if run.current_room.is_depth() || run.phase != Phase::Fighting {
        return;
    }

    let Ok((player_transform, mut facing, mut sprite, mut anim)) = player_query.single_mut() else {
        return;
    };
    if !mouse.pressed(MouseButton::Left) || !run.swing_timer.is_finished() {
        return;
    }

    let player_pos = player_transform.translation.truncate();

    let aim = cursor_world(&windows, &camera_query)
        .map(|world| world - player_pos)
        .filter(|v| v.length_squared() > 1.0)
        .map(|v| v.normalize())
        .unwrap_or(facing.0);
    let aim = if aim.length_squared() > 0.0 {
        aim.normalize()
    } else {
        Vec2::X
    };

    // Store the mouse aim direction and reset its timer
    last_aim.direction = Some(aim);
    last_aim.timer = Timer::from_seconds(0.3, TimerMode::Once);

    facing.0 = aim;
    if aim.x.abs() > 0.05 {
        sprite.flip_x = aim.x < 0.0;
    }

    anim.start_attack();
    if let Some(atlas) = sprite.texture_atlas.as_mut() {
        atlas.index = KNIGHT_ATTACK.0;
    }

    let sword = run.sword;
    let base_damage = run.swing_damage();

    // Determine if this is a critical hit
    let is_crit = rand::random::<f32>() < sword.crit_chance;
    let damage = if is_crit {
        (base_damage as f32 * 1.5).ceil() as i32
    } else {
        base_damage
    };

    let angle = aim.y.atan2(aim.x);
    let center = player_pos + aim * (sword.reach * 0.45);
    let visual = sword.reach * 1.7;

    commands.spawn((
        art.image_sprite(&art.slash, Vec2::splat(visual), sword.color),
        Transform {
            translation: center.extend(3.2),
            rotation: Quat::from_rotation_z(angle),
            ..default()
        },
        Slash {
            damage,
            reach: sword.reach,
            arc: sword.arc,
            knockback: sword.knockback,
            lifesteal: sword.lifesteal,
            origin: player_pos,
            dir: aim,
            life: Timer::from_seconds(0.18, TimerMode::Once),
            hit: HashSet::new(),
            slash_style: sword.slash_style,
            is_crit,
        },
        RoomEntity,
    ));

    if sword.slash_wave {
        spawn_projectile(
            &mut commands,
            &art,
            player_pos + aim * 30.0,
            aim * run.projectile_speed,
            ProjectileOwner::Player,
            damage,
        );
    }

    play(&mut commands, &sfx.swing, 0.3);
    run.swing_timer.reset();
}

/// World-space position of the mouse cursor, if it is over the window.
pub fn cursor_world(
    windows: &Query<&Window>,
    camera_query: &Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let window = windows.iter().next()?;
    let cursor = window.cursor_position()?;
    let (camera, camera_transform) = camera_query.iter().next()?;
    camera.viewport_to_world_2d(camera_transform, cursor).ok()
}
