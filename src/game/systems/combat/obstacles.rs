use crate::game::assets::GameArt;
use crate::game::components::*;
use crate::game::systems::effects::spawn_impact_particles;
use crate::game::systems::player::play;
use crate::game::assets::Sfx;
use bevy::prelude::*;

/// System to handle slash hits on destructible obstacles.
pub fn slash_hit_obstacles(
    mut commands: Commands,
    art: Res<GameArt>,
    sfx: Res<Sfx>,
    mut slashes: Query<&mut Slash>,
    mut obstacles: Query<(Entity, &Transform, &mut Obstacle, &mut Sprite), With<DestructibleObstacle>>,
) {
    for mut slash in &mut slashes {
        for (entity, transform, mut obstacle, mut sprite) in &mut obstacles {
            if slash.hit_obstacles.contains(&entity) {
                continue;
            }

            let to = transform.translation.truncate() - slash.origin;
            let dist = to.length();
            if dist <= obstacle.radius + slash.reach {
                obstacle.hp -= slash.damage;
                slash.hit_obstacles.insert(entity);

                if obstacle.hp <= 0 {
                    // Destroy obstacle
                    spawn_destruction_particles(&mut commands, &art, transform.translation.truncate());
                    commands.entity(entity).despawn();
                } else {
                    // Visual damage feedback - make slightly transparent
                    sprite.color = sprite.color.with_alpha(0.7);
                    // Spawn hit particles
                    spawn_impact_particles(&mut commands, &art, transform.translation.truncate(), HitIntensity::Medium);
                }
            }
        }
    }
}

/// System to handle projectile hits on destructible obstacles.
pub fn projectile_hit_obstacles(
    mut commands: Commands,
    art: Res<GameArt>,
    mut obstacles: Query<(Entity, &Transform, &mut Obstacle, &mut Sprite), With<DestructibleObstacle>>,
    projectiles: Query<(Entity, &Transform, &Projectile), With<Projectile>>,
) {
    for (proj_entity, proj_transform, projectile) in &projectiles {
        let proj_pos = proj_transform.translation.truncate();

        for (obs_entity, obs_transform, mut obstacle, mut sprite) in &mut obstacles {
            let obs_pos = obs_transform.translation.truncate();
            let dist = proj_pos.distance(obs_pos);

            if dist <= obstacle.radius + 15.0 { // 15.0 is projectile radius approximation
                obstacle.hp -= projectile.damage;

                if obstacle.hp <= 0 {
                    // Destroy obstacle
                    spawn_destruction_particles(&mut commands, &art, obs_pos);
                    commands.entity(obs_entity).despawn();
                } else {
                    // Visual damage feedback
                    sprite.color = sprite.color.with_alpha(0.7);
                    spawn_impact_particles(&mut commands, &art, obs_pos, HitIntensity::Light);
                }

                // Despawn projectile
                commands.entity(proj_entity).despawn();
                break;
            }
        }
    }
}

/// Spawn destruction particles when an obstacle is destroyed.
fn spawn_destruction_particles(commands: &mut Commands, art: &GameArt, pos: Vec2) {
    for i in 0..8 {
        let angle = (i as f32 / 8.0) * std::f32::consts::TAU;
        let vel = Vec2::new(angle.cos(), angle.sin()) * 80.0;
        commands.spawn((
            art.image_sprite(&art.orb, Vec2::splat(12.0), Color::srgb(0.55, 0.35, 0.2)),
            Transform::from_translation(pos.extend(4.0)),
            Particle {
                velocity: vel,
                life: Timer::from_seconds(0.3, TimerMode::Once),
            },
            RoomEntity,
        ));
    }
}