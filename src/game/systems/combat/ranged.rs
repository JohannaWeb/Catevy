use crate::game::assets::{GameArt, Sfx};
use crate::game::components::*;
use crate::game::state::{Phase, RunState, ScreenShake};
use bevy::prelude::*;

use super::damage::spawn_damage_number;
use crate::game::systems::player::{ARENA_HALF_WIDTH, ARENA_HALF_HEIGHT, play};

#[allow(clippy::too_many_arguments)]
pub fn update_projectiles(
    time: Res<Time>,
    mut commands: Commands,
    mut run: ResMut<RunState>,
    mut shake: ResMut<ScreenShake>,
    mut hitstop: ResMut<HitStop>,
    art: Res<GameArt>,
    sfx: Res<Sfx>,
    obstacles: Query<(&Transform, &Obstacle), Without<Projectile>>,
    mut queries: ParamSet<(
        Query<(Entity, &mut Transform, &mut Projectile), With<Projectile>>,
        Query<(Entity, &Transform, &mut Enemy), With<Enemy>>,
        Query<&Transform, With<Player>>,
    )>,
) {
    let delta = time.delta_secs();
    let player_query = queries.p2();
    let Ok(player_transform) = player_query.single() else { return; };
    let player_pos = player_transform.translation.truncate();
    let enemy_snapshots: Vec<(Entity, Vec2)> = queries.p1().iter()
        .map(|(entity, transform, _)| (entity, transform.translation.truncate()))
        .collect();
    let mut enemy_hits: Vec<(Entity, i32, Vec2)> = Vec::new();

    for (entity, mut transform, mut projectile) in queries.p0().iter_mut() {
        projectile.lifetime.tick(time.delta());
        if projectile.lifetime.is_finished() {
            commands.entity(entity).despawn();
            continue;
        }

        let velocity = projectile.velocity * delta;
        transform.translation.x += velocity.x;
        transform.translation.y += velocity.y;

        let position = transform.translation.truncate();
        if position.x.abs() > ARENA_HALF_WIDTH + 32.0 || position.y.abs() > ARENA_HALF_HEIGHT + 32.0 {
            commands.entity(entity).despawn();
            continue;
        }

        let mut hit_obstacle = false;
        for (obs_transform, obstacle) in &obstacles {
            let obs_pos = obs_transform.translation.truncate();
            if position.distance(obs_pos) < obstacle.radius {
                hit_obstacle = true;
                break;
            }
        }
        if hit_obstacle {
            commands.entity(entity).despawn();
            continue;
        }

        match projectile.owner {
            ProjectileOwner::Player => {
                for (enemy_entity, enemy_pos) in &enemy_snapshots {
                    if position.distance(*enemy_pos) > 24.0 { continue; }
                    enemy_hits.push((*enemy_entity, projectile.damage, *enemy_pos));
                    commands.entity(entity).despawn();
                    break;
                }
            }
            ProjectileOwner::Enemy => {
                if position.distance(player_pos) <= 22.0 {
                    if run.invuln.is_finished() {
                        run.player_hp = (run.player_hp - projectile.damage).clamp(0, run.player_max_hp);
                        shake.trauma = (shake.trauma + 0.35).min(1.0);
                        play(&mut commands, &sfx.hurt, 0.5);
                        hitstop.0 = hitstop.0.max(0.05);
                        if run.player_hp == 0 { run.phase = Phase::GameOver; }
                    }
                    commands.entity(entity).despawn();
                }
            }
        }
    }

    for (target, damage, pos) in enemy_hits {
        if let Ok((_, _, mut enemy)) = queries.p1().get_mut(target) {
            enemy.hp -= damage;
            play(&mut commands, &sfx.hit, 0.3);
            spawn_damage_number(&mut commands, &art.font, pos + Vec2::new(0.0, 24.0), damage);
            if enemy.hp > 0 { commands.entity(target).insert(HitFlash::new()); }
        }
    }
}