use crate::game::ability::AbilitySlot;
use crate::game::assets::{GameArt, Sfx};
use crate::game::components::*;
use crate::game::state::{Phase, RunState};
use crate::game::sword::SWORDS;
use bevy::audio::{AudioSource, PlaybackSettings, Volume};
use bevy::prelude::*;

use crate::game::systems::effects::spawn_pickup_pop;

pub fn collect_pickups(
    mut commands: Commands,
    mut run: ResMut<RunState>,
    art: Res<GameArt>,
    sfx: Res<Sfx>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    pickups: Query<(Entity, &Transform, &Pickup)>,
) {
    if run.phase == Phase::GameOver { return; }

    let Ok(player_transform) = player_query.single() else { return; };
    let player_pos = player_transform.translation.truncate();

    for (entity, transform, pickup) in &pickups {
        let pos = transform.translation.truncate();
        if player_pos.distance(pos) > 30.0 { continue; }

        match pickup.kind {
            PickupKind::Heal(amount) => {
                run.player_hp = (run.player_hp + amount).min(run.player_max_hp);
            }
            PickupKind::DamageUp(amount) => {
                run.damage_bonus += amount;
            }
            PickupKind::SpeedUp(amount) => {
                run.player_speed += amount as f32;
            }
            PickupKind::MaxHpUp(amount) => {
                run.player_max_hp += amount;
                run.player_hp += amount;
            }
            PickupKind::SwordDrop(index) => {
                run.equip(SWORDS[index]);
            }
            PickupKind::AbilityDrop(ability) => {
                if run.abilities.iter().any(|s| s.ability == ability) {
                    run.player_hp = (run.player_hp + 3).min(run.player_max_hp);
                } else if run.abilities.len() < 3 {
                    run.abilities.push(AbilitySlot::new(ability));
                } else {
                    run.abilities[2] = AbilitySlot::new(ability);
                }
            }
        }
        commands.entity(entity).despawn();
        spawn_pickup_pop(&mut commands, &art, pos, pickup.kind);
        play(&mut commands, &sfx.pickup, 0.5);
    }
}

fn play(commands: &mut Commands, clip: &Handle<AudioSource>, volume: f32) {
    commands.spawn((
        AudioPlayer::new(clip.clone()),
        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(volume)),
    ));
}

pub fn bob_pickups(time: Res<Time>, mut query: Query<(&mut Transform, &mut Bob)>) {
    for (mut transform, mut bob) in &mut query {
        bob.phase += time.delta_secs() * bob.speed;
        transform.translation.y = bob.base_y + bob.phase.sin() * bob.amplitude;
    }
}