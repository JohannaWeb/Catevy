use crate::game::assets::{GameArt, Sfx};
use crate::game::components::*;
use crate::game::state::{RunState, ScreenShake};
use crate::game::sword::SlashStyle;
use crate::game::systems::effects::{
    spawn_flame_particles, spawn_impact_particles, spawn_leaf_particles, spawn_spark_particles,
};
use bevy::prelude::*;

use super::damage::{spawn_crit_damage_number, spawn_damage_number};
use super::enemy_hurt_radius;
use crate::game::systems::player::{ARENA_HALF_HEIGHT, ARENA_HALF_WIDTH, play};

const SLASH_HIT_PADDING: f32 = 10.0;

pub fn update_slashes(
    time: Res<Time>,
    mut commands: Commands,
    mut run: ResMut<RunState>,
    mut shake: ResMut<ScreenShake>,
    art: Res<GameArt>,
    sfx: Res<Sfx>,
    mut slashes: Query<(Entity, &mut Slash, &mut Sprite, &mut Transform), Without<Enemy>>,
    mut enemies: Query<(Entity, &mut Transform, &mut Enemy), (With<Enemy>, Without<Slash>)>,
    mut hitstop: ResMut<HitStop>,
) {
    for (entity, mut slash, mut sprite, mut transform) in &mut slashes {
        slash.life.tick(time.delta());
        if slash.life.is_finished() {
            commands.entity(entity).despawn();
            continue;
        }

        let frac = slash.life.fraction();
        sprite.color = sprite.color.with_alpha((1.0 - frac).clamp(0.0, 1.0));
        transform.scale = Vec3::splat(1.0 + frac * 0.2);

        for (enemy_entity, mut enemy_transform, mut enemy) in &mut enemies {
            if slash.hit.contains(&enemy_entity) {
                continue;
            }
            let to = enemy_transform.translation.truncate() - slash.origin;
            let dist = to.length();

            let enemy_radius = enemy_hurt_radius(enemy.kind, enemy.elite);
            if !slash_hits_enemy(&slash, to, dist, enemy_radius) {
                continue;
            }

            slash.hit.insert(enemy_entity);
            enemy.hp -= slash.damage;
            let enemy_pos = enemy_transform.translation.truncate();

            // Calculate hit intensity based on damage percentage
            let intensity = HitIntensity::from_damage(slash.damage, enemy.max_hp);

            // Apply intensity-scaled effects
            hitstop.0 = hitstop.0.max(intensity.hitstop_duration());
            shake.trauma = (shake.trauma + intensity.shake_trauma()).min(1.0);

            // Spawn impact particles
            spawn_impact_particles(&mut commands, &art, enemy_pos, intensity);

            // Spawn weapon-specific particles
            match slash.slash_style {
                SlashStyle::Flame => {
                    spawn_flame_particles(&mut commands, &art, enemy_pos, slash.dir);
                }
                SlashStyle::Spark => {
                    spawn_spark_particles(&mut commands, &art, enemy_pos);
                }
                SlashStyle::Leaf => {
                    spawn_leaf_particles(&mut commands, &art, enemy_pos, slash.dir);
                }
                SlashStyle::Standard | SlashStyle::Trail => {
                    // Standard slashes don't need additional particles
                }
            }

            play(&mut commands, &sfx.hit, 0.3);

            // Spawn crit or normal damage number
            if slash.is_crit {
                spawn_crit_damage_number(
                    &mut commands,
                    &art.font,
                    enemy_pos + Vec2::new(0.0, 24.0),
                    slash.damage,
                );
            } else {
                spawn_damage_number(
                    &mut commands,
                    &art.font,
                    enemy_pos + Vec2::new(0.0, 24.0),
                    slash.damage,
                );
            }

            let push_dir = if dist > 1.0 { to / dist } else { slash.dir };
            let push = push_dir * slash.knockback * 0.18;
            enemy_transform.translation.x =
                (enemy_transform.translation.x + push.x).clamp(-ARENA_HALF_WIDTH, ARENA_HALF_WIDTH);
            enemy_transform.translation.y = (enemy_transform.translation.y + push.y)
                .clamp(-ARENA_HALF_HEIGHT, ARENA_HALF_HEIGHT);

            if enemy.hp <= 0 {
                if slash.lifesteal > 0 {
                    run.player_hp = (run.player_hp + slash.lifesteal).min(run.player_max_hp);
                }
            } else {
                commands.entity(enemy_entity).insert(HitFlash::new());
                // Don't apply knockback animation to bosses (they have their own scale animations)
                if enemy.kind != EnemyKind::Boss {
                    commands.entity(enemy_entity).insert(Knockback {
                        duration: Timer::from_seconds(0.15, TimerMode::Once),
                        direction: push_dir,
                    });
                }
            }
        }
    }
}

fn slash_hits_enemy(slash: &Slash, to_enemy: Vec2, distance: f32, enemy_radius: f32) -> bool {
    if distance <= enemy_radius + SLASH_HIT_PADDING {
        return true;
    }

    if distance > slash.reach + enemy_radius + SLASH_HIT_PADDING {
        return false;
    }

    let angular_padding = ((enemy_radius + SLASH_HIT_PADDING) / distance)
        .asin()
        .min(0.75);
    let dot_threshold = (slash.arc + angular_padding).cos();
    slash.dir.dot(to_enemy / distance) >= dot_threshold
}
