use crate::game::assets::{GameArt, Sfx};
use crate::game::components::*;
use crate::game::spawning::{spawn_enemy_kind, spawn_gem_reward};
use crate::game::state::{Phase, RunState, ScreenShake};
use bevy::prelude::*;

use crate::game::systems::player::{play, spawn_poof};

#[allow(clippy::too_many_arguments)]
pub fn resolve_enemy_deaths(
    mut commands: Commands,
    art: Res<GameArt>,
    sfx: Res<Sfx>,
    mut run: ResMut<RunState>,
    mut shake: ResMut<ScreenShake>,
    mut hitstop: ResMut<HitStop>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    enemies: Query<(Entity, &Transform, &Enemy)>,
) {
    let player_pos = player_query.single().map(|t| t.translation.truncate()).unwrap_or(Vec2::ZERO);
    let mut player_hp = run.player_hp;
    let invuln = !run.invuln.is_finished();
    let mut kills_this_frame = 0u32;

    for (entity, transform, enemy) in &enemies {
        if enemy.hp > 0 { continue; }
        kills_this_frame += 1;
        let pos = transform.translation.truncate();
        spawn_poof(&mut commands, &art, pos);
        play(&mut commands, &sfx.enemy_death, 0.45);
        hitstop.0 = hitstop.0.max(0.04);

        if enemy.elite {
            let roll = pos.x.abs() as u32 + pos.y.abs() as u32 + run.floor;
            spawn_gem_reward(&mut commands, &art, pos, roll);
        }
        if enemy.splits {
            for i in 0..2 {
                let off = Vec2::new(if i == 0 { 32.0 } else { -32.0 }, 0.0);
                spawn_enemy_kind(&mut commands, &art, EnemyKind::Kitten, run.floor, pos + off, false);
            }
        }
        if enemy.explodes {
            explode(&mut commands, &art, pos, enemy.damage + 2, 120.0, player_pos, &mut player_hp, &mut shake, invuln);
            play(&mut commands, &sfx.explosion, 0.6);
        }
        commands.entity(entity).despawn();
    }

    if kills_this_frame > 0 {
        run.combo_count += kills_this_frame;
        run.combo_timer.reset();
        run.best_combo = run.best_combo.max(run.combo_count);
    }

    run.player_hp = player_hp.clamp(0, run.player_max_hp);
    if run.player_hp == 0 { run.phase = Phase::GameOver; }
}

#[allow(clippy::too_many_arguments)]
pub fn explode(
    commands: &mut Commands,
    art: &GameArt,
    pos: Vec2,
    damage: i32,
    radius: f32,
    player_pos: Vec2,
    player_hp: &mut i32,
    shake: &mut ScreenShake,
    invuln: bool,
) {
    if !invuln && player_pos.distance(pos) <= radius {
        *player_hp -= damage;
        shake.trauma = (shake.trauma + 0.6).min(1.0);
    }
    commands.spawn((
        art.image_sprite(&art.orb, Vec2::splat(radius * 2.0), Color::srgb(1.0, 0.6, 0.3)),
        Transform::from_translation(pos.extend(3.6)).with_scale(Vec3::splat(0.25)),
        Explosion { life: Timer::from_seconds(0.3, TimerMode::Once), max_scale: 1.0 },
        RoomEntity,
    ));
}

pub fn update_explosions(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut Sprite, &mut Explosion)>,
) {
    for (entity, mut transform, mut sprite, mut explosion) in &mut query {
        explosion.life.tick(time.delta());
        if explosion.life.is_finished() {
            commands.entity(entity).despawn();
            continue;
        }
        let f = explosion.life.fraction();
        transform.scale = Vec3::splat((0.3 + 0.7 * f) * explosion.max_scale);
        sprite.color = sprite.color.with_alpha(1.0 - f);
    }
}