use crate::game::ability::{ABILITY_KEY_LABELS};
use crate::game::components::*;
use crate::game::state::{Phase, RunState};
use bevy::prelude::*;

/// Tracks the last combo milestone reached for visual feedback.
#[derive(Resource, Default)]
pub struct ComboState {
    pub last_milestone: u32,
}

pub fn update_boss_health_bar(
    boss_query: Query<(&Enemy, &BossPhases)>,
    depth_boss_query: Query<&DepthBoss>,
    mut root_query: Query<&mut Visibility, With<BossHealthBarRoot>>,
    mut fill_query: Query<(&mut Node, &mut BackgroundColor), With<BossHealthBarFill>>,
    mut text_query: Query<&mut Text, With<BossHealthBarText>>,
) {
    if let Some(boss) = depth_boss_query.iter().find(|boss| boss.hp > 0) {
        update_boss_bar(
            boss.name,
            boss.hp.clamp(0, boss.max_hp),
            boss.max_hp,
            &mut root_query,
            &mut fill_query,
            &mut text_query,
        );
        return;
    }

    let boss = boss_query.iter().find(|(enemy, _)| enemy.hp > 0);
    let Some((enemy, phases)) = boss else {
        for mut visibility in &mut root_query {
            *visibility = Visibility::Hidden;
        }
        return;
    };

    let hp = enemy.hp.clamp(0, enemy.max_hp);
    let label = format!(
        "{} - Phase {}",
        boss_name(phases.boss_type),
        phases.current_phase
    );
    update_boss_bar(
        &label,
        hp,
        enemy.max_hp,
        &mut root_query,
        &mut fill_query,
        &mut text_query,
    );
}

fn update_boss_bar(
    name: &str,
    hp: i32,
    max_hp: i32,
    root_query: &mut Query<&mut Visibility, With<BossHealthBarRoot>>,
    fill_query: &mut Query<(&mut Node, &mut BackgroundColor), With<BossHealthBarFill>>,
    text_query: &mut Query<&mut Text, With<BossHealthBarText>>,
) {
    let fraction = hp as f32 / max_hp.max(1) as f32;
    for mut visibility in root_query.iter_mut() {
        *visibility = Visibility::Visible;
    }
    for (mut node, mut color) in fill_query.iter_mut() {
        node.width = Val::Percent((fraction * 100.0).clamp(0.0, 100.0));
        color.0 = if fraction <= 0.30 {
            Color::srgb(1.0, 0.46, 0.18)
        } else {
            Color::srgb(0.86, 0.16, 0.20)
        };
    }
    for mut text in text_query.iter_mut() {
        text.0 = format!("{name}  {hp}/{max_hp}");
    }
}

fn boss_name(boss_type: BossType) -> &'static str {
    match boss_type {
        BossType::GoblinKing => "Goblin King",
        BossType::Necromancer => "Necromancer",
        BossType::Dragon => "Dragon",
    }
}

pub fn update_hud(game: Res<RunState>, mut query: Query<&mut Text, With<HudText>>) {
    if !game.is_changed() { return; }

    let Ok(mut text) = query.single_mut() else { return; };
    if game.current_room.is_depth() {
        text.0 = match game.phase {
            Phase::Fighting => format!(
                "Floor {} - Room {}/{}   ({:?})\nHP: {}/{}\n\nThe world has unfolded.\nMove: WASD\nSlash: hold left click\nDash/skills stay banked for 2D rooms",
                game.floor,
                game.room,
                game.rooms_per_floor,
                game.current_room,
                game.player_hp,
                game.player_max_hp,
            ),
            Phase::RoomCleared => format!(
                "The Depth Gate is open.\nRoom type: {:?}\n\nMove to the blue gate and press E.",
                game.current_room,
            ),
        };
        return;
    }

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
        Phase::Fighting => {
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
        Phase::RoomCleared => format!(
            "Room cleared.\nRoom type: {:?}\n\nSword: {} ({})\n\nPress E at the door to continue.",
            game.current_room, game.sword.name, game.sword.quirk,
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
