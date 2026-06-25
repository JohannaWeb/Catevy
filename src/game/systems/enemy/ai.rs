use crate::game::assets::{GameArt, Sfx};
use crate::game::components::*;
use crate::game::state::{Phase, RunState, ScreenShake};
use bevy::prelude::*;

use super::behavior::{chase_velocity, trigger_range, windup_time, action_cooldown};
use super::attacks::perform_attack;
use super::telegraphs::spawn_telegraph;
use crate::game::systems::player::{ARENA_HALF_WIDTH, ARENA_HALF_HEIGHT, play};

#[allow(clippy::too_many_arguments)]
pub fn enemy_ai(
    time: Res<Time>,
    mut run: ResMut<RunState>,
    mut shake: ResMut<ScreenShake>,
    mut hitstop: ResMut<HitStop>,
    art: Res<GameArt>,
    sfx: Res<Sfx>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    mut enemies: Query<(Entity, &mut Transform, &mut Enemy, &mut Sprite, &mut CatAnimation), (With<Enemy>, Without<Player>)>,
    mut commands: Commands,
) {
    if run.phase != Phase::Fighting { return; }

    let Ok(player_transform) = player_query.single() else { return; };
    let player_pos = player_transform.translation.truncate();
    let delta = time.delta_secs();
    let mut player_hp = run.player_hp;
    let invuln = !run.invuln.is_finished();

    for (entity, mut transform, mut enemy, mut sprite, mut anim) in &mut enemies {
        if enemy.hp <= 0 { continue; }
        let pos = transform.translation.truncate();
        let offset = player_pos - pos;
        let distance = offset.length();
        let dir = if distance > 0.1 { offset / distance } else { Vec2::X };

        if offset.x.abs() > 0.05 && enemy.state != EnemyState::Charge {
            sprite.flip_x = offset.x < 0.0;
        }

        enemy.action_cd.tick(time.delta());
        enemy.state_timer.tick(time.delta());

        let scale = if enemy.state == EnemyState::Windup {
            1.0 + 0.3 * enemy.state_timer.fraction()
        } else { 1.0 };
        transform.scale = Vec3::splat(scale);

        match enemy.state {
            EnemyState::Chase => {
                anim.moving = true;
                let step = chase_velocity(enemy.kind, dir, distance) * enemy.speed * delta;
                move_clamped(&mut transform, step);

                if distance <= trigger_range(enemy.kind) && enemy.action_cd.is_finished() {
                    enemy.charge_dir = dir;
                    enemy.state = EnemyState::Windup;
                    enemy.state_timer = Timer::from_seconds(windup_time(enemy.kind), TimerMode::Once);
                    enemy.has_telegraph = false;
                }
            }
            EnemyState::Windup => {
                anim.moving = false;
                if !enemy.has_telegraph {
                    spawn_telegraph(&mut commands, &art, pos, enemy.kind, enemy.charge_dir, windup_time(enemy.kind));
                    enemy.has_telegraph = true;
                }

                if enemy.state_timer.is_finished() {
                    enemy.has_telegraph = false;
                    perform_attack(&mut commands, &art, &mut enemy, pos, player_pos, distance, dir, &mut player_hp, &mut shake, entity, run.floor, invuln, &sfx);
                    anim.start_attack();
                    let recover = if enemy.kind == EnemyKind::Charger {
                        enemy.state = EnemyState::Charge;
                        enemy.state_timer = Timer::from_seconds(0.45, TimerMode::Once);
                        continue;
                    } else { 0.5 };
                    enemy.state = EnemyState::Recover;
                    enemy.state_timer = Timer::from_seconds(recover, TimerMode::Once);
                }
            }
            EnemyState::Charge => {
                anim.moving = true;
                let step = enemy.charge_dir * (enemy.speed * 3.4) * delta;
                let before = transform.translation.truncate();
                move_clamped(&mut transform, step);
                let after = transform.translation.truncate();
                if player_pos.distance(after) < 46.0 {
                    if !invuln {
                        player_hp -= enemy.damage;
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
                    enemy.action_cd = Timer::from_seconds(action_cooldown(enemy.kind), TimerMode::Once);
                }
            }
        }
    }

    let clamped = player_hp.clamp(0, run.player_max_hp);
    if clamped < run.player_hp {
        play(&mut commands, &sfx.hurt, 0.5);
        hitstop.0 = hitstop.0.max(0.06);
    }
    run.player_hp = clamped;
    if run.player_hp == 0 { run.phase = Phase::GameOver; }
}

fn move_clamped(transform: &mut Transform, step: Vec2) {
    transform.translation.x = (transform.translation.x + step.x).clamp(-ARENA_HALF_WIDTH, ARENA_HALF_WIDTH);
    transform.translation.y = (transform.translation.y + step.y).clamp(-ARENA_HALF_HEIGHT, ARENA_HALF_HEIGHT);
}