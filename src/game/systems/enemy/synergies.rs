use crate::game::assets::GameArt;
use crate::game::components::*;
use bevy::prelude::*;

pub fn enemy_synergies(
    time: Res<Time>,
    art: Res<GameArt>,
    mut commands: Commands,
    mut synergies: Query<(Entity, &Transform, &mut EnemySynergy)>,
    mut enemies: Query<(Entity, &Transform, &mut Enemy, &mut Sprite), Without<EnemySynergy>>,
) {
    for (_entity, transform, mut synergy) in &mut synergies {
        synergy.timer.tick(time.delta());
        if !synergy.timer.is_finished() { continue; }

        let pos = transform.translation.truncate();
        synergy.timer.reset();

        match synergy.kind {
            SynergyKind::Healer => {
                for (_, ally_transform, mut ally, _) in &mut enemies {
                    let ally_pos = ally_transform.translation.truncate();
                    if pos.distance(ally_pos) < 120.0 && ally.hp < ally.max_hp {
                        ally.hp = (ally.hp + 1).min(ally.max_hp);
                        commands.spawn((
                            art.image_sprite(&art.orb, Vec2::splat(10.0), Color::srgb(0.3, 1.0, 0.5)),
                            Transform::from_translation(ally_pos.extend(4.0)),
                            Particle { velocity: Vec2::new(0.0, 40.0), life: Timer::from_seconds(0.4, TimerMode::Once) },
                            RoomEntity,
                        ));
                        break;
                    }
                }
            }
            SynergyKind::Pack => {}
            SynergyKind::Commander => {}
        }
    }
}