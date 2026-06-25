use crate::game::assets::{GameArt, Sfx};
use crate::game::components::*;
use crate::game::state::RunState;
use bevy::prelude::*;

use super::damage::spawn_damage_number;
use crate::game::systems::player::{ARENA_HALF_WIDTH, ARENA_HALF_HEIGHT, play};

pub fn update_slashes(
    time: Res<Time>,
    mut commands: Commands,
    mut run: ResMut<RunState>,
    art: Res<GameArt>,
    sfx: Res<Sfx>,
    mut slashes: Query<(Entity, &mut Slash, &mut Sprite, &mut Transform), Without<Enemy>>,
    mut enemies: Query<(Entity, &mut Transform, &mut Enemy), (With<Enemy>, Without<Slash>)>,
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

        let cos_arc = slash.arc.cos();
        for (enemy_entity, mut enemy_transform, mut enemy) in &mut enemies {
            if slash.hit.contains(&enemy_entity) { continue; }
            let to = enemy_transform.translation.truncate() - slash.origin;
            let dist = to.length();
            if dist < 1.0 || dist > slash.reach + 22.0 { continue; }
            if slash.dir.dot(to / dist) < cos_arc { continue; }

            slash.hit.insert(enemy_entity);
            enemy.hp -= slash.damage;
            let enemy_pos = enemy_transform.translation.truncate();
            play(&mut commands, &sfx.hit, 0.3);
            spawn_damage_number(&mut commands, &art.font, enemy_pos + Vec2::new(0.0, 24.0), slash.damage);
            let push = (to / dist) * slash.knockback * 0.18;
            enemy_transform.translation.x = (enemy_transform.translation.x + push.x).clamp(-ARENA_HALF_WIDTH, ARENA_HALF_WIDTH);
            enemy_transform.translation.y = (enemy_transform.translation.y + push.y).clamp(-ARENA_HALF_HEIGHT, ARENA_HALF_HEIGHT);

            if enemy.hp <= 0 {
                if slash.lifesteal > 0 {
                    run.player_hp = (run.player_hp + slash.lifesteal).min(run.player_max_hp);
                }
            } else {
                commands.entity(enemy_entity).insert(HitFlash::new());
            }
        }
    }
}