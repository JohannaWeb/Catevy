use crate::game::assets::{GameArt, Sfx};
use crate::game::components::*;
use crate::game::state::{GameState, Phase, RunState, ScreenShake};
use bevy::prelude::*;
use bevy::time::Real;

use super::attacks::perform_attack;
use super::behavior::{action_cooldown, chase_velocity, trigger_range, windup_time};
use super::telegraphs::spawn_telegraph;
use crate::game::systems::combat::{
    PLAYER_HURT_RADIUS, reset_hurt_invuln, spawn_damage_number_colored,
};
use crate::game::systems::player::{ARENA_HALF_HEIGHT, ARENA_HALF_WIDTH, play};

const GOBLIN_KING_WINDUP_SECONDS: f32 = 0.55;

#[allow(clippy::too_many_arguments)]
pub fn enemy_ai(
    time: Res<Time>,
    real_time: Res<Time<Real>>,
    mut run: ResMut<RunState>,
    mut shake: ResMut<ScreenShake>,
    mut hitstop: ResMut<HitStop>,
    mut next_state: ResMut<NextState<GameState>>,
    art: Res<GameArt>,
    sfx: Res<Sfx>,
    player_query: Query<(Entity, &Transform), (With<Player>, Without<Enemy>)>,
    mut enemies: Query<
        (
            Entity,
            &mut Transform,
            &mut Enemy,
            &mut Sprite,
            &mut CatAnimation,
            Option<&mut BossPhases>,
        ),
        (With<Enemy>, Without<Player>, Without<Knockback>),
    >,
    mut commands: Commands,
) {
    if run.current_room.is_depth() || run.phase != Phase::Fighting {
        return;
    }

    let Ok((player_entity, player_transform)) = player_query.single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();
    let delta = time.delta_secs();
    let mut player_hp = run.player_hp;
    let mut invuln = !run.invuln.is_finished();

    for (entity, mut transform, mut enemy, mut sprite, mut anim, boss_phases) in &mut enemies {
        if enemy.hp <= 0 {
            continue;
        }
        let pos = transform.translation.truncate();
        let offset = player_pos - pos;
        let distance = offset.length();
        let dir = if distance > 0.1 {
            offset / distance
        } else {
            Vec2::X
        };
        let mut boss_type = None;
        if let Some(mut phases) = boss_phases {
            boss_type = Some(phases.boss_type);
            let next_phase = boss_phase(enemy.hp, enemy.max_hp);
            if next_phase != phases.current_phase {
                phases.current_phase = next_phase;
                spawn_boss_phase_pulse(&mut commands, &art, pos);
                shake.trauma = (shake.trauma + 0.18).min(1.0);
                hitstop.0 = hitstop.0.max(0.04);
            }
        }

        if offset.x.abs() > 0.05 && enemy.state != EnemyState::Charge {
            sprite.flip_x = offset.x < 0.0;
        }

        // Use real time so hitstop (which zeros virtual time) can't freeze the state machine.
        enemy.action_cd.tick(real_time.delta());
        enemy.state_timer.tick(real_time.delta());

        let scale = if enemy.state == EnemyState::Windup {
            1.0 + 0.3 * enemy.state_timer.fraction()
        } else {
            1.0
        };
        transform.scale = Vec3::splat(scale);

        // All bosses use special AI
        if enemy.kind == EnemyKind::Boss {
            match enemy.state {
                EnemyState::Windup => {
                    anim.moving = false;
                    if !enemy.has_telegraph {
                        // Spawn telegraph for all bosses
                        spawn_telegraph(
                            &mut commands,
                            &art,
                            pos,
                            enemy.kind,
                            enemy.charge_dir,
                            GOBLIN_KING_WINDUP_SECONDS,
                        );
                        enemy.has_telegraph = true;
                    }

                    if enemy.state_timer.is_finished() {
                        enemy.has_telegraph = false;
                        let attack_dir = normalized_or_x(enemy.charge_dir);
                        perform_attack(
                            &mut commands,
                            &art,
                            &mut enemy,
                            pos,
                            player_pos,
                            distance,
                            attack_dir,
                            &mut player_hp,
                            &mut shake,
                            entity,
                            run.floor,
                            invuln,
                            &sfx,
                            boss_type,
                        );
                        anim.start_attack();
                        enemy.state = EnemyState::Recover;
                        enemy.state_timer = Timer::from_seconds(0.18, TimerMode::Once);
                    }
                }
                EnemyState::Recover => {
                    anim.moving = false;
                    if enemy.state_timer.is_finished() {
                        enemy.state = EnemyState::Chase;
                        let cooldown = match boss_type {
                            Some(BossType::GoblinKing) => goblin_king_cooldown(&enemy),
                            Some(BossType::Necromancer) => necromancer_cooldown(&enemy),
                            _ => dragon_cooldown(&enemy),
                        };
                        enemy.action_cd = Timer::from_seconds(cooldown, TimerMode::Once);
                    }
                }
                _ => {
                    anim.moving = true;
                    let step = match boss_type {
                        Some(BossType::GoblinKing) => {
                            goblin_king_movement(pos, dir, distance, time.elapsed_secs())
                                * enemy.speed
                                * delta
                        }
                        _ => {
                            // Generic boss movement - strafe and approach
                            let strafe_side = if (time.elapsed_secs() * 0.75).sin() >= 0.0 {
                                1.0
                            } else {
                                -1.0
                            };
                            let strafe =
                                Vec2::new(-dir.y, dir.x) * strafe_side * 0.6;
                            let approach = if distance > 400.0 {
                                dir * 0.5
                            } else if distance < 250.0 {
                                -dir * 0.4
                            } else {
                                Vec2::ZERO
                            };
                            (strafe + approach).normalize_or_zero() * enemy.speed * delta
                        }
                    };
                    move_clamped(&mut transform, step);

                    // Only attack if player is in range
                    let attack_range = match boss_type {
                        Some(BossType::GoblinKing) => 760.0,
                        _ => 450.0, // Other bosses have similar range
                    };

                    if distance <= attack_range && enemy.action_cd.is_finished() {
                        enemy.charge_dir = dir;
                        enemy.state = EnemyState::Windup;
                        enemy.state_timer =
                            Timer::from_seconds(GOBLIN_KING_WINDUP_SECONDS, TimerMode::Once);
                        enemy.has_telegraph = false;
                    }
                }
            }
            continue;
        }

        match enemy.state {
            EnemyState::Chase => {
                anim.moving = true;
                let step = chase_velocity(enemy.kind, dir, distance) * enemy.speed * delta;
                move_clamped(&mut transform, step);

                if distance <= trigger_range(enemy.kind) && enemy.action_cd.is_finished() {
                    enemy.charge_dir = dir;
                    enemy.state = EnemyState::Windup;
                    enemy.state_timer =
                        Timer::from_seconds(windup_time(enemy.kind), TimerMode::Once);
                    enemy.has_telegraph = false;
                }
            }
            EnemyState::Windup => {
                anim.moving = false;
                if !enemy.has_telegraph {
                    spawn_telegraph(
                        &mut commands,
                        &art,
                        pos,
                        enemy.kind,
                        enemy.charge_dir,
                        windup_time(enemy.kind),
                    );
                    enemy.has_telegraph = true;
                }

                if enemy.state_timer.is_finished() {
                    enemy.has_telegraph = false;
                    let before_hp = player_hp;
                    perform_attack(
                        &mut commands,
                        &art,
                        &mut enemy,
                        pos,
                        player_pos,
                        distance,
                        dir,
                        &mut player_hp,
                        &mut shake,
                        entity,
                        run.floor,
                        invuln,
                        &sfx,
                        boss_type,
                    );
                    if player_hp < before_hp {
                        invuln = true;
                        reset_hurt_invuln(&mut run.invuln);
                    }
                    anim.start_attack();
                    // Charger: set charge state here (perform_attack is empty for Charger).
                    // Chonker: perform_attack already set state=Charge + charge_dir, just continue.
                    if enemy.kind == EnemyKind::Charger {
                        enemy.state = EnemyState::Charge;
                        enemy.state_timer = Timer::from_seconds(0.45, TimerMode::Once);
                        continue;
                    }
                    if enemy.kind == EnemyKind::Chonker {
                        continue;
                    }
                    enemy.state = EnemyState::Recover;
                    enemy.state_timer = Timer::from_seconds(0.5, TimerMode::Once);
                }
            }
            EnemyState::Charge => {
                anim.moving = true;
                let step = enemy.charge_dir * (enemy.speed * 3.4) * delta;
                let before = transform.translation.truncate();
                move_clamped(&mut transform, step);
                let after = transform.translation.truncate();
                if player_pos.distance(after) < PLAYER_HURT_RADIUS + 24.0 {
                    if !invuln {
                        player_hp -= enemy.damage;
                        invuln = true;
                        reset_hurt_invuln(&mut run.invuln);
                        shake.trauma = (shake.trauma + 0.5).min(1.0);
                    }
                    enemy.state = EnemyState::Recover;
                    enemy.state_timer = Timer::from_seconds(0.7, TimerMode::Once);
                } else if enemy.state_timer.is_finished() || before.distance(after) < 0.5 {
                    enemy.state = EnemyState::Recover;
                    enemy.state_timer = Timer::from_seconds(0.7, TimerMode::Once);
                }
            }
            EnemyState::Recover => {
                anim.moving = false;
                if enemy.state_timer.is_finished() {
                    enemy.state = EnemyState::Chase;
                    let cooldown = if enemy.kind == EnemyKind::Boss
                        && matches!(boss_type, Some(BossType::GoblinKing))
                    {
                        1.75
                    } else {
                        action_cooldown(enemy.kind)
                    };
                    enemy.action_cd = Timer::from_seconds(cooldown, TimerMode::Once);
                }
            }
        }
    }

    let clamped = player_hp.clamp(0, run.player_max_hp);
    if clamped < run.player_hp {
        let damage_taken = run.player_hp - clamped;
        play(&mut commands, &sfx.hurt, 0.5);
        commands.entity(player_entity).insert(HitFlash::new());
        spawn_damage_number_colored(
            &mut commands,
            &art.font,
            player_pos + Vec2::new(0.0, 34.0),
            damage_taken,
            Color::srgb(1.0, 0.35, 0.32),
        );
        hitstop.0 = hitstop.0.max(0.06);
    }
    run.player_hp = clamped;
    if run.player_hp == 0 {
        next_state.set(GameState::GameOver);
    }
}

/// Tick state timers for enemies currently in knockback — excluded from enemy_ai by query
/// filter, but their state machine must keep advancing so rapid hits can't freeze them.
pub fn tick_knockback_enemy_timers(
    real_time: Res<Time<Real>>,
    mut enemies: Query<&mut Enemy, (With<Enemy>, Without<Player>, With<Knockback>)>,
) {
    for mut enemy in &mut enemies {
        if enemy.hp <= 0 {
            continue;
        }
        enemy.action_cd.tick(real_time.delta());
        enemy.state_timer.tick(real_time.delta());
    }
}

fn goblin_king_cooldown(enemy: &Enemy) -> f32 {
    let hp_percent = enemy.hp as f32 / enemy.max_hp.max(1) as f32;
    if hp_percent > 0.66 {
        1.55
    } else if hp_percent > 0.33 {
        1.3
    } else {
        1.1
    }
}

fn necromancer_cooldown(enemy: &Enemy) -> f32 {
    let hp_percent = enemy.hp as f32 / enemy.max_hp.max(1) as f32;
    if hp_percent > 0.66 {
        1.8 // Slower while summoning
    } else if hp_percent > 0.33 {
        1.3
    } else {
        0.9 // Fast projectile spam at low HP
    }
}

fn dragon_cooldown(enemy: &Enemy) -> f32 {
    let hp_percent = enemy.hp as f32 / enemy.max_hp.max(1) as f32;
    if hp_percent > 0.66 {
        1.4
    } else if hp_percent > 0.33 {
        1.1
    } else {
        0.8
    }
}

fn goblin_king_movement(pos: Vec2, dir_to_player: Vec2, distance: f32, elapsed: f32) -> Vec2 {
    let strafe_side = if (elapsed * 0.75).sin() >= 0.0 { 1.0 } else { -1.0 };
    let strafe = Vec2::new(-dir_to_player.y, dir_to_player.x) * strafe_side * 0.75;
    let range = if distance < 260.0 {
        -dir_to_player * 1.1
    } else if distance > 430.0 {
        dir_to_player * 0.65
    } else {
        Vec2::ZERO
    };
    let center_pull = Vec2::new(-pos.x / ARENA_HALF_WIDTH, -pos.y / ARENA_HALF_HEIGHT) * 0.35;
    (strafe + range + center_pull).normalize_or_zero() * 0.72
}

fn normalized_or_x(dir: Vec2) -> Vec2 {
    if dir.length_squared() > 0.0 {
        dir.normalize()
    } else {
        Vec2::X
    }
}

fn boss_phase(hp: i32, max_hp: i32) -> u8 {
    let hp_percent = hp as f32 / max_hp.max(1) as f32;
    if hp_percent > 0.66 {
        1
    } else if hp_percent > 0.33 {
        2
    } else {
        3
    }
}

fn spawn_boss_phase_pulse(commands: &mut Commands, art: &GameArt, pos: Vec2) {
    commands.spawn((
        art.image_sprite(
            &art.orb,
            Vec2::splat(180.0),
            Color::srgba(1.0, 0.22, 0.28, 0.42),
        ),
        Transform::from_translation(pos.extend(3.7)).with_scale(Vec3::splat(0.25)),
        Explosion {
            life: Timer::from_seconds(0.38, TimerMode::Once),
            max_scale: 1.0,
        },
        RoomEntity,
    ));
}

fn move_clamped(transform: &mut Transform, step: Vec2) {
    transform.translation.x =
        (transform.translation.x + step.x).clamp(-ARENA_HALF_WIDTH, ARENA_HALF_WIDTH);
    transform.translation.y =
        (transform.translation.y + step.y).clamp(-ARENA_HALF_HEIGHT, ARENA_HALF_HEIGHT);
}
