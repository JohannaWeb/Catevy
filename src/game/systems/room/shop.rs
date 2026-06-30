use crate::game::ability::AbilitySlot;
use crate::game::assets::{GameArt, Sfx};
use crate::game::components::*;
use crate::game::state::{GameState, PersistentState, Phase, RunState};
use crate::game::sword::SWORDS;
use bevy::audio::{AudioSource, PlaybackSettings, Volume};
use bevy::prelude::*;

pub fn shop_item_interact(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut run: ResMut<RunState>,
    mut persistent: ResMut<PersistentState>,
    game_state: Res<State<GameState>>,
    art: Res<GameArt>,
    sfx: Res<Sfx>,
    player_query: Query<&Transform, With<Player>>,
    shop_items: Query<(Entity, &Transform, &ShopItemMarker)>,
) {
    if *game_state.get() != GameState::Playing || run.phase != Phase::RoomCleared {
        return;
    }

    if !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }

    let Ok(player_transform) = player_query.single() else { return; };
    let player_pos = player_transform.translation.truncate();

    for (entity, transform, item) in &shop_items {
        let pos = transform.translation.truncate();
        if player_pos.distance(pos) <= 50.0 && persistent.currency >= item.cost {
            // Deduct currency
            persistent.currency -= item.cost;

            // Apply the shop item effect
            apply_shop_item(&mut run, &item.kind);

            // Despawn the item
            commands.entity(entity).despawn();

            // Play pickup sound
            play(&mut commands, &sfx.pickup, 0.6);

            // Save persistent state
            persistent.save();

            // Only buy one item per key press
            break;
        }
    }
}

fn apply_shop_item(run: &mut RunState, kind: &ShopItemKind) {
    match kind {
        ShopItemKind::HealFull => {
            run.player_hp = run.player_max_hp;
        }
        ShopItemKind::DamageUp(amount) => {
            run.damage_bonus += amount;
        }
        ShopItemKind::SpeedUp(amount) => {
            run.player_speed += *amount as f32;
        }
        ShopItemKind::MaxHpUp(amount) => {
            run.player_max_hp += amount;
            run.player_hp += amount;
        }
        ShopItemKind::Sword(idx) => {
            if *idx < SWORDS.len() {
                run.equip(SWORDS[*idx].clone());
            }
        }
        ShopItemKind::Ability(ability) => {
            if run.abilities.iter().any(|s| s.ability == *ability) {
                // Already have it, heal instead
                run.player_hp = (run.player_hp + 3).min(run.player_max_hp);
            } else if run.abilities.len() < 3 {
                run.abilities.push(AbilitySlot::new(*ability));
            } else {
                // Replace last ability
                run.abilities[2] = AbilitySlot::new(*ability);
            }
        }
        ShopItemKind::Reroll => {
            // Reroll would need to be implemented with shop state tracking
            // For now, just give a small bonus
            run.player_hp = (run.player_hp + 2).min(run.player_max_hp);
        }
    }
}

fn play(commands: &mut Commands, clip: &Handle<AudioSource>, volume: f32) {
    commands.spawn((
        AudioPlayer::new(clip.clone()),
        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(volume)),
    ));
}