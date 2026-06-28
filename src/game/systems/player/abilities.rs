use crate::game::ability::{ABILITY_KEYS, Ability};
use crate::game::assets::{GameArt, KNIGHT_IDLE};
use crate::game::components::*;
use crate::game::spawning::spawn_projectile;
use crate::game::state::{RunState, ScreenShake};
use crate::game::sword::SlashStyle;
use bevy::prelude::*;
use std::collections::HashSet;

use super::combat::{ARENA_HALF_HEIGHT, ARENA_HALF_WIDTH, play};

#[allow(clippy::too_many_arguments)]
pub fn use_abilities(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut run: ResMut<RunState>,
    mut shake: ResMut<ScreenShake>,
    art: Res<GameArt>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    sfx: Res<crate::game::assets::Sfx>,
    mut player_query: Query<(&mut Transform, &Facing), (With<Player>, Without<Enemy>)>,
    mut enemies: Query<&mut Transform, (With<Enemy>, Without<Player>)>,
    mut commands: Commands,
) {
    run.invuln.tick(time.delta());
    let slot_count = run.abilities.len();
    for i in 0..slot_count {
        run.abilities[i].cd.tick(time.delta());
    }
    if run.current_room.is_depth() {
        return;
    }
    if run.phase != crate::game::state::Phase::Fighting {
        return;
    }

    let Ok((mut player_transform, facing)) = player_query.single_mut() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    let aim = super::combat::cursor_world(&windows, &camera_query)
        .map(|w| w - player_pos)
        .filter(|v| v.length_squared() > 1.0)
        .map(|v| v.normalize())
        .unwrap_or(facing.0);

    let mut move_dir = Vec2::ZERO;
    if keyboard.pressed(KeyCode::KeyW) {
        move_dir.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        move_dir.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        move_dir.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        move_dir.x += 1.0;
    }
    let dash_dir = if move_dir.length_squared() > 0.0 {
        move_dir.normalize()
    } else {
        facing.0
    };

    let swing_damage = run.swing_damage();
    let max_hp = run.player_max_hp;
    let projectile_speed = run.projectile_speed;

    for i in 0..slot_count {
        if !keyboard.just_pressed(ABILITY_KEYS[i]) || !run.abilities[i].cd.is_finished() {
            continue;
        }
        let ability = run.abilities[i].ability;
        match ability {
            Ability::Dash => {
                let target = player_pos + dash_dir * 150.0;
                let clamped_target = Vec2::new(
                    target.x.clamp(-ARENA_HALF_WIDTH, ARENA_HALF_WIDTH),
                    target.y.clamp(-ARENA_HALF_HEIGHT, ARENA_HALF_HEIGHT),
                );

                // Spawn 5 afterimages along dash path
                for i in 1..=5 {
                    let t = i as f32 / 6.0;
                    let afterimage_pos = player_pos.lerp(clamped_target, t);
                    commands.spawn((
                        art.knight_sprite(KNIGHT_IDLE.0, 3.4, Color::srgba(0.5, 0.9, 1.0, 0.6)),
                        Transform::from_translation(afterimage_pos.extend(2.5)),
                        Afterimage {
                            life: Timer::from_seconds(0.15, TimerMode::Once),
                        },
                        RoomEntity,
                    ));
                }

                player_transform.translation.x = clamped_target.x;
                player_transform.translation.y = clamped_target.y;
                run.invuln = Timer::from_seconds(0.3, TimerMode::Once);
                spawn_poof(&mut commands, &art, player_pos);
            }
            Ability::Whirlwind => {
                spawn_aoe_slash(
                    &mut commands,
                    &art,
                    player_pos,
                    std::f32::consts::PI,
                    150.0,
                    swing_damage + 2,
                    160.0,
                    ability.color(),
                );
            }
            Ability::WarCry => {
                for mut enemy_transform in &mut enemies {
                    let to = enemy_transform.translation.truncate() - player_pos;
                    let dist = to.length();
                    if dist > 0.1 && dist < 230.0 {
                        let push = to / dist * 170.0;
                        enemy_transform.translation.x = (enemy_transform.translation.x + push.x)
                            .clamp(-ARENA_HALF_WIDTH, ARENA_HALF_WIDTH);
                        enemy_transform.translation.y = (enemy_transform.translation.y + push.y)
                            .clamp(-ARENA_HALF_HEIGHT, ARENA_HALF_HEIGHT);
                    }
                }
                run.invuln = Timer::from_seconds(0.6, TimerMode::Once);
                shake.trauma = (shake.trauma + 0.4).min(1.0);
                spawn_ring(&mut commands, &art, player_pos, 230.0, ability.color());
            }
            Ability::Hairball => {
                for k in 0..5 {
                    let a = (k as f32 - 2.0) * 0.18;
                    let dir = rotate(aim, a);
                    spawn_projectile(
                        &mut commands,
                        &art,
                        player_pos + dir * 26.0,
                        dir * projectile_speed,
                        ProjectileOwner::Player,
                        swing_damage,
                    );
                }
            }
            Ability::SecondWind => {
                run.player_hp = (run.player_hp + 6).min(max_hp);
                run.invuln = Timer::from_seconds(0.8, TimerMode::Once);
                spawn_ring(&mut commands, &art, player_pos, 100.0, ability.color());
            }
            Ability::Zoomies => {
                run.player_speed *= 3.0;
                run.invuln = Timer::from_seconds(2.0, TimerMode::Once);
                spawn_poof(&mut commands, &art, player_pos);
            }
            Ability::Purr => {
                for mut enemy_transform in &mut enemies {
                    let enemy_pos = enemy_transform.translation.truncate();
                    if player_pos.distance(enemy_pos) < 150.0 {
                        let to = enemy_pos - player_pos;
                        let push = to.normalize_or_zero() * 100.0;
                        enemy_transform.translation.x = (enemy_transform.translation.x + push.x)
                            .clamp(-ARENA_HALF_WIDTH, ARENA_HALF_WIDTH);
                        enemy_transform.translation.y = (enemy_transform.translation.y + push.y)
                            .clamp(-ARENA_HALF_HEIGHT, ARENA_HALF_HEIGHT);
                    }
                }
                spawn_ring(&mut commands, &art, player_pos, 150.0, ability.color());
            }
            Ability::CatNap => {
                run.player_hp = (run.player_hp + 2).min(max_hp);
                run.invuln = Timer::from_seconds(3.0, TimerMode::Once);
                spawn_ring(&mut commands, &art, player_pos, 60.0, ability.color());
            }
        }
        let sound = match ability {
            Ability::Dash => &sfx.dash,
            Ability::WarCry => &sfx.explosion,
            Ability::SecondWind => &sfx.pickup,
            _ => &sfx.swing,
        };
        play(&mut commands, sound, 0.45);
        run.abilities[i].cd = Timer::from_seconds(ability.cooldown(), TimerMode::Once);
    }
}

/// Rotate a unit vector by `radians`.
pub fn rotate(v: Vec2, radians: f32) -> Vec2 {
    let (s, c) = radians.sin_cos();
    Vec2::new(v.x * c - v.y * s, v.x * s + v.y * c)
}

/// A 360°-ish damaging shockwave (used by Whirlwind), drawn as a fading disc.
pub fn spawn_aoe_slash(
    commands: &mut Commands,
    art: &GameArt,
    origin: Vec2,
    arc: f32,
    reach: f32,
    damage: i32,
    knockback: f32,
    color: Color,
) {
    commands.spawn((
        art.image_sprite(&art.orb, Vec2::splat(reach * 2.0), color),
        Transform::from_translation(origin.extend(3.2)),
        Slash {
            damage,
            reach,
            arc,
            knockback,
            lifesteal: 0,
            origin,
            dir: Vec2::X,
            life: Timer::from_seconds(0.22, TimerMode::Once),
            hit: HashSet::new(),
            slash_style: SlashStyle::Standard,
            is_crit: false,
        },
        RoomEntity,
    ));
}

/// A purely-visual expanding ring (used by War Cry / Second Wind).
pub fn spawn_ring(commands: &mut Commands, art: &GameArt, pos: Vec2, size: f32, color: Color) {
    commands.spawn((
        art.image_sprite(&art.orb, Vec2::splat(size * 2.0), color.with_alpha(0.7)),
        Transform::from_translation(pos.extend(3.5)).with_scale(Vec3::splat(0.2)),
        Explosion {
            life: Timer::from_seconds(0.3, TimerMode::Once),
            max_scale: 1.0,
        },
        RoomEntity,
    ));
}

/// A small burst of dust particles.
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
