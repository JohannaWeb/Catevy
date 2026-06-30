use crate::game::assets::{GameArt, Sfx};
use crate::game::components::*;
use crate::game::state::{GameState, RunState, ScreenShake};
use crate::game::systems::effects::spawn_impact_particles;
use bevy::prelude::*;

use super::damage::{spawn_damage_number, spawn_damage_number_colored};
use super::{
    ENEMY_PROJECTILE_RADIUS, PLAYER_HURT_RADIUS, PLAYER_PROJECTILE_RADIUS, enemy_hurt_radius,
    reset_hurt_invuln, segment_intersects_circle,
};
use crate::game::systems::player::{ARENA_HALF_HEIGHT, ARENA_HALF_WIDTH, play};

#[allow(clippy::too_many_arguments)]
pub fn update_projectiles(
    time: Res<Time>,
    mut commands: Commands,
    mut run: ResMut<RunState>,
    mut shake: ResMut<ScreenShake>,
    mut hitstop: ResMut<HitStop>,
    mut next_state: ResMut<NextState<GameState>>,
    art: Res<GameArt>,
    sfx: Res<Sfx>,
    obstacles: Query<(&Transform, &Obstacle), Without<Projectile>>,
    mut queries: ParamSet<(
        Query<(Entity, &mut Transform, &mut Projectile), With<Projectile>>,
        Query<(Entity, &Transform, &mut Enemy), With<Enemy>>,
        Query<(Entity, &Transform), With<Player>>,
    )>,
) {
    let delta = time.delta_secs();
    let player_query = queries.p2();
    let Ok((player_entity, player_transform)) = player_query.single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();
    let enemy_snapshots: Vec<(Entity, Vec2, f32)> = queries
        .p1()
        .iter()
        .map(|(entity, transform, enemy)| {
            (
                entity,
                transform.translation.truncate(),
                enemy_hurt_radius(enemy.kind, enemy.elite),
            )
        })
        .collect();
    let mut enemy_hits: Vec<(Entity, i32, Vec2)> = Vec::new();

    for (entity, mut transform, mut projectile) in queries.p0().iter_mut() {
        projectile.lifetime.tick(time.delta());
        if projectile.lifetime.is_finished() {
            commands.entity(entity).despawn();
            continue;
        }

        let previous_position = transform.translation.truncate();
        let velocity = projectile.velocity * delta;
        transform.translation.x += velocity.x;
        transform.translation.y += velocity.y;

        let position = transform.translation.truncate();
        if position.x.abs() > ARENA_HALF_WIDTH + 32.0 || position.y.abs() > ARENA_HALF_HEIGHT + 32.0
        {
            commands.entity(entity).despawn();
            continue;
        }

        let mut hit_obstacle = false;
        for (obs_transform, obstacle) in &obstacles {
            let obs_pos = obs_transform.translation.truncate();
            let projectile_radius = match projectile.owner {
                ProjectileOwner::Player => PLAYER_PROJECTILE_RADIUS,
                ProjectileOwner::Enemy => ENEMY_PROJECTILE_RADIUS,
            };
            if segment_intersects_circle(
                previous_position,
                position,
                obs_pos,
                obstacle.radius + projectile_radius,
            ) {
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
                for (enemy_entity, enemy_pos, enemy_radius) in &enemy_snapshots {
                    let hit_radius = *enemy_radius + PLAYER_PROJECTILE_RADIUS;
                    if !segment_intersects_circle(
                        previous_position,
                        position,
                        *enemy_pos,
                        hit_radius,
                    ) {
                        continue;
                    }
                    enemy_hits.push((*enemy_entity, projectile.damage, *enemy_pos));
                    commands.entity(entity).despawn();
                    break;
                }
            }
            ProjectileOwner::Enemy => {
                let hit_radius = PLAYER_HURT_RADIUS + ENEMY_PROJECTILE_RADIUS;
                if segment_intersects_circle(previous_position, position, player_pos, hit_radius) {
                    if run.invuln.is_finished() {
                        run.player_hp =
                            (run.player_hp - projectile.damage).clamp(0, run.player_max_hp);
                        reset_hurt_invuln(&mut run.invuln);
                        shake.trauma = (shake.trauma + 0.35).min(1.0);
                        play(&mut commands, &sfx.hurt, 0.5);
                        commands.entity(player_entity).insert(HitFlash::new());
                        spawn_damage_number_colored(
                            &mut commands,
                            &art.font,
                            player_pos + Vec2::new(0.0, 34.0),
                            projectile.damage,
                            Color::srgb(1.0, 0.35, 0.32),
                        );
                        hitstop.0 = hitstop.0.max(0.05);
                        if run.player_hp == 0 {
                            next_state.set(GameState::GameOver);
                        }
                    }
                    commands.entity(entity).despawn();
                }
            }
        }
    }

    for (target, damage, pos) in enemy_hits {
        if let Ok((_, _, mut enemy)) = queries.p1().get_mut(target) {
            enemy.hp -= damage;

            // Calculate hit intensity
            let intensity = HitIntensity::from_damage(damage, enemy.max_hp);
            hitstop.0 = hitstop.0.max(intensity.hitstop_duration());
            shake.trauma = (shake.trauma + intensity.shake_trauma() * 0.5).min(1.0);

            // Spawn impact particles
            spawn_impact_particles(&mut commands, &art, pos, intensity);

            play(&mut commands, &sfx.hit, 0.3);
            spawn_damage_number(&mut commands, &art.font, pos + Vec2::new(0.0, 24.0), damage);
            if enemy.hp > 0 {
                commands.entity(target).insert(HitFlash::new());
            }
        }
    }
}
