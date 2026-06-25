use crate::game::ability::{ABILITY_KEY_LABELS};
use crate::game::components::*;
use crate::game::state::RunState;
use bevy::prelude::*;

pub fn update_hud(game: Res<RunState>, mut query: Query<&mut Text, With<HudText>>) {
    if !game.is_changed() { return; }

    let Ok(mut text) = query.single_mut() else { return; };
    let mut skills = String::new();
    for (i, slot) in game.abilities.iter().enumerate() {
        let status = if slot.cd.is_finished() {
            "ready".to_string()
        } else {
            format!("{:.1}s", slot.cd.remaining_secs())
        };
        skills.push_str(&format!("\n  [{}] {} ({})", ABILITY_KEY_LABELS[i], slot.ability.name(), status));
    }

    text.0 = match game.phase {
        crate::game::state::Phase::Fighting => {
            let combo_text = if game.combo_count > 2 {
                format!("\nCOMBO x{}! (+{} dmg)", game.combo_count, game.combo_damage_bonus())
            } else { String::new() };
            format!(
                "Floor {} - Room {}/{}   ({:?})\nHP: {}/{}\n\nSword: {}  (dmg {})\n  {}\n\nSkills:{}{}\n\nSwing: hold left click (aim w/ mouse)\nMove: WASD",
                game.floor, game.room, game.rooms_per_floor, game.current_room,
                game.player_hp, game.player_max_hp,
                game.sword.name, game.swing_damage(), game.sword.quirk,
                skills, combo_text,
            )
        }
        crate::game::state::Phase::RoomCleared => format!(
            "Room cleared.\nRoom type: {:?}\n\nSword: {} ({})\n\nPress E at the door to continue.",
            game.current_room, game.sword.name, game.sword.quirk,
        ),
        crate::game::state::Phase::GameOver => format!(
            "The clan fell on floor {}.\nPress R to try again.\n\nFinal sword: {}\nHP: {}/{}\nSwing damage: {}\nBest combo: {}",
            game.floor, game.sword.name, game.player_hp, game.player_max_hp, game.swing_damage(), game.best_combo,
        ),
    };
}

pub fn combo_decay(time: Res<Time>, mut run: ResMut<RunState>) {
    if run.combo_count == 0 { return; }
    run.combo_timer.tick(time.delta());
    if run.combo_timer.is_finished() {
        run.combo_count = 0;
    }
}

pub fn tick_modifiers(mut run: ResMut<RunState>) {
    run.active_modifiers.retain(|(_, remaining)| *remaining > 0);
}