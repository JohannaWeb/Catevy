use crate::game::assets::{GameArt, Sfx};
use crate::game::components::*;
use crate::game::progression::advance_room;
use crate::game::spawning::{despawn_room, spawn_room};
use crate::game::state::{Phase, RunState};
use bevy::prelude::*;

use super::combat::reset_hurt_invuln;
use super::player::play;

const DEPTH_ARENA_HALF: f32 = 6.0;

pub fn sync_dimension_view(
    run: Res<RunState>,
    mut camera_2d: Query<&mut Camera, (With<Main2dCamera>, Without<DepthCamera>)>,
    mut camera_3d: Query<&mut Camera, (With<DepthCamera>, Without<Main2dCamera>)>,
    mut player_2d: Query<&mut Visibility, With<Player>>,
) {
    let depth = run.current_room.is_depth();
    for mut camera in &mut camera_2d {
        camera.is_active = !depth;
    }
    for mut camera in &mut camera_3d {
        camera.is_active = depth;
    }
    for mut visibility in &mut player_2d {
        *visibility = if depth {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
    }
}

pub fn depth_camera_follow(
    run: Res<RunState>,
    player: Query<&Transform, With<DepthPlayer>>,
    mut camera: Query<&mut Transform, (With<DepthCamera>, Without<DepthPlayer>)>,
) {
    if !run.current_room.is_depth() {
        return;
    }
    let Ok(player_transform) = player.single() else {
        return;
    };
    let Ok(mut camera_transform) = camera.single_mut() else {
        return;
    };
    let target = player_transform.translation;
    camera_transform.translation = target + Vec3::new(0.0, 8.5, 10.5);
    camera_transform.look_at(target, Vec3::Y);
}

pub fn depth_player_movement(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    run: Res<RunState>,
    mut player: Query<(&mut Transform, &mut DepthPlayer)>,
) {
    if !run.current_room.is_depth() || run.phase == Phase::GameOver {
        return;
    }
    let Ok((mut transform, mut depth_player)) = player.single_mut() else {
        return;
    };

    let mut direction = Vec3::ZERO;
    if keyboard.pressed(KeyCode::KeyW) {
        direction.z -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        direction.z += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }

    if direction.length_squared() <= 0.0 {
        return;
    }

    let dir = direction.normalize();
    depth_player.facing = dir;
    let speed = run.effective_speed() / 58.0;
    transform.translation += dir * speed * time.delta_secs();
    transform.translation.x = transform.translation.x.clamp(-DEPTH_ARENA_HALF, DEPTH_ARENA_HALF);
    transform.translation.z = transform.translation.z.clamp(-DEPTH_ARENA_HALF, DEPTH_ARENA_HALF);
}

pub fn depth_player_combat(
    mouse: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut run: ResMut<RunState>,
    sfx: Res<Sfx>,
    mut player: Query<(&Transform, &mut DepthPlayer)>,
    mut bosses: Query<(&mut DepthBoss, &Transform)>,
) {
    if !run.current_room.is_depth() || run.phase != Phase::Fighting {
        return;
    }
    let Ok((player_transform, mut depth_player)) = player.single_mut() else {
        return;
    };
    if !mouse.pressed(MouseButton::Left) || !run.swing_timer.is_finished() {
        return;
    }

    let player_pos = player_transform.translation;
    let mut facing = depth_player.facing;
    let mut hit = false;
    let damage = run.swing_damage();
    for (mut boss, boss_transform) in &mut bosses {
        if boss.hp <= 0 {
            continue;
        }
        let to_boss = boss_transform.translation - player_pos;
        let flat = Vec3::new(to_boss.x, 0.0, to_boss.z);
        if flat.length_squared() > 0.01 {
            facing = flat.normalize();
            depth_player.facing = facing;
        }
        if flat.length() <= 2.25 {
            boss.hp -= damage;
            hit = true;
        }
    }

    let slash_pos = player_pos + facing * 0.75 + Vec3::Y * 0.15;
    let yaw = facing.z.atan2(facing.x);
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.6, 0.08, 0.35))),
        MeshMaterial3d(materials.add(Color::srgb(1.0, 0.90, 0.40))),
        Transform {
            translation: slash_pos,
            rotation: Quat::from_rotation_y(-yaw),
            ..default()
        },
        DepthSlash {
            life: Timer::from_seconds(0.14, TimerMode::Once),
        },
        RoomEntity,
    ));

    play(&mut commands, if hit { &sfx.hit } else { &sfx.swing }, 0.35);
    run.swing_timer.reset();
}

pub fn depth_boss_ai(
    time: Res<Time>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut run: ResMut<RunState>,
    sfx: Res<Sfx>,
    player: Query<&Transform, (With<DepthPlayer>, Without<DepthBoss>)>,
    mut bosses: Query<(&mut Transform, &mut DepthBoss), Without<DepthPlayer>>,
) {
    if !run.current_room.is_depth() || run.phase != Phase::Fighting {
        return;
    }
    let Ok(player_transform) = player.single() else {
        return;
    };
    let player_pos = player_transform.translation;

    for (mut transform, mut boss) in &mut bosses {
        if boss.hp <= 0 {
            continue;
        }
        boss.attack_timer.tick(time.delta());

        let offset = player_pos - transform.translation;
        let flat = Vec3::new(offset.x, 0.0, offset.z);
        let distance = flat.length();
        let dir = if distance > 0.01 {
            flat / distance
        } else {
            Vec3::Z
        };

        if distance > 1.9 {
            transform.translation += dir * boss.speed * time.delta_secs();
            transform.translation.x = transform.translation.x.clamp(-5.5, 5.5);
            transform.translation.z = transform.translation.z.clamp(-5.5, 5.5);
        } else if run.invuln.is_finished() {
            run.player_hp = (run.player_hp - boss.damage).clamp(0, run.player_max_hp);
            reset_hurt_invuln(&mut run.invuln);
            play(&mut commands, &sfx.hurt, 0.45);
        }

        if boss.attack_timer.is_finished() {
            let spreads: &[f32] = if boss.final_boss {
                &[-0.35, 0.0, 0.35]
            } else {
                &[0.0]
            };
            for angle in spreads {
                let shot_dir = rotate_xz(dir, *angle);
                commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(0.28, 0.28, 0.28))),
                    MeshMaterial3d(materials.add(Color::srgb(1.0, 0.28, 0.34))),
                    Transform::from_translation(transform.translation + shot_dir * 0.9),
                    DepthProjectile {
                        velocity: shot_dir * if boss.final_boss { 5.0 } else { 4.3 },
                        damage: boss.damage,
                        lifetime: Timer::from_seconds(2.4, TimerMode::Once),
                    },
                    RoomEntity,
                ));
            }
            boss.attack_timer = Timer::from_seconds(if boss.final_boss { 1.15 } else { 1.55 }, TimerMode::Once);
        }
    }

    if run.player_hp == 0 {
        run.phase = Phase::GameOver;
    }
}

pub fn update_depth_projectiles(
    time: Res<Time>,
    mut commands: Commands,
    mut run: ResMut<RunState>,
    sfx: Res<Sfx>,
    player: Query<&Transform, (With<DepthPlayer>, Without<DepthProjectile>)>,
    mut projectiles: Query<(Entity, &mut Transform, &mut DepthProjectile), Without<DepthPlayer>>,
) {
    if !run.current_room.is_depth() || run.phase != Phase::Fighting {
        return;
    }
    let Ok(player_transform) = player.single() else {
        return;
    };
    let player_pos = player_transform.translation;

    for (entity, mut transform, mut projectile) in &mut projectiles {
        projectile.lifetime.tick(time.delta());
        if projectile.lifetime.is_finished() {
            commands.entity(entity).despawn();
            continue;
        }

        transform.translation += projectile.velocity * time.delta_secs();
        if transform.translation.x.abs() > DEPTH_ARENA_HALF + 1.0
            || transform.translation.z.abs() > DEPTH_ARENA_HALF + 1.0
        {
            commands.entity(entity).despawn();
            continue;
        }

        let delta = transform.translation - player_pos;
        let flat_distance = Vec2::new(delta.x, delta.z).length();
        if flat_distance <= 0.65 {
            if run.invuln.is_finished() {
                run.player_hp = (run.player_hp - projectile.damage).clamp(0, run.player_max_hp);
                reset_hurt_invuln(&mut run.invuln);
                play(&mut commands, &sfx.hurt, 0.45);
                if run.player_hp == 0 {
                    run.phase = Phase::GameOver;
                }
            }
            commands.entity(entity).despawn();
        }
    }
}

pub fn update_depth_slashes(
    time: Res<Time>,
    mut commands: Commands,
    mut slashes: Query<(Entity, &mut DepthSlash)>,
) {
    for (entity, mut slash) in &mut slashes {
        slash.life.tick(time.delta());
        if slash.life.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn depth_room_progression(
    mut commands: Commands,
    mut run: ResMut<RunState>,
    mut exits: Query<(&mut DepthExit, &mut Visibility)>,
    bosses: Query<(Entity, &DepthBoss)>,
) {
    if !run.current_room.is_depth() || run.phase != Phase::Fighting {
        return;
    }

    let mut alive = false;
    for (entity, boss) in &bosses {
        if boss.hp > 0 {
            alive = true;
        } else {
            commands.entity(entity).despawn();
        }
    }
    if alive {
        return;
    }

    run.phase = Phase::RoomCleared;
    for (mut exit, mut visibility) in &mut exits {
        exit.active = true;
        *visibility = Visibility::Visible;
    }
}

pub fn depth_exit_interact(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut run: ResMut<RunState>,
    art: Res<GameArt>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player: Query<&Transform, With<DepthPlayer>>,
    exits: Query<(&Transform, &DepthExit)>,
    room_entities: Query<Entity, With<RoomEntity>>,
) {
    if !run.current_room.is_depth()
        || run.phase != Phase::RoomCleared
        || !keyboard.just_pressed(KeyCode::KeyE)
    {
        return;
    }
    let Ok(player_transform) = player.single() else {
        return;
    };
    let player_pos = player_transform.translation;
    let near_exit = exits.iter().any(|(transform, exit)| {
        exit.active && {
            let delta = transform.translation - player_pos;
            Vec2::new(delta.x, delta.z).length() <= 1.6
        }
    });
    if !near_exit {
        return;
    }

    despawn_room(&mut commands, &room_entities);
    advance_room(&mut run);
    spawn_room(&mut commands, &art, &mut meshes, &mut materials, &mut run);
}

pub fn debug_jump_to_depth(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut run: ResMut<RunState>,
    art: Res<GameArt>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    room_entities: Query<Entity, With<RoomEntity>>,
) {
    if !keyboard.just_pressed(KeyCode::F3) {
        return;
    }

    despawn_room(&mut commands, &room_entities);
    run.floor = 3;
    run.room = 1;
    run.phase = Phase::Fighting;
    spawn_room(&mut commands, &art, &mut meshes, &mut materials, &mut run);
}

fn rotate_xz(v: Vec3, radians: f32) -> Vec3 {
    let (s, c) = radians.sin_cos();
    Vec3::new(v.x * c - v.z * s, 0.0, v.x * s + v.z * c).normalize_or_zero()
}
