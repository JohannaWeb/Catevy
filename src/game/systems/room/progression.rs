use crate::game::assets::GameArt;
use crate::game::components::*;
use crate::game::progression::advance_room;
use crate::game::spawning::{despawn_room, spawn_door, spawn_room, spawn_sword_reward};
use crate::game::state::{PersistentState, Phase, RoomKind, RunState};
use crate::game::sword::SWORDS;
use bevy::prelude::*;

use crate::game::systems::player::DOOR_DISTANCE;

pub fn process_room_clear(
    mut commands: Commands,
    mut run: ResMut<RunState>,
    art: Res<GameArt>,
    enemies: Query<Entity, (With<Enemy>, Without<Player>)>,
    existing_door: Query<Entity, With<Door>>,
) {
    if run.phase != Phase::Fighting || !enemies.is_empty() { return; }

    run.phase = Phase::RoomCleared;
    if existing_door.is_empty() {
        spawn_door(&mut commands, &art);
        if run.current_room == RoomKind::Boss {
            let index = 1 + (run.floor as usize) % (SWORDS.len() - 1);
            spawn_sword_reward(&mut commands, &art, index);
        }
    }
}

pub fn door_interact(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut run: ResMut<RunState>,
    art: Res<GameArt>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    door_query: Query<(Entity, &Transform), With<Door>>,
    room_entities: Query<Entity, With<RoomEntity>>,
) {
    if run.phase != Phase::RoomCleared || !keyboard.just_pressed(KeyCode::KeyE) { return; }

    let Ok(player_transform) = player_query.single() else { return; };
    let player_pos = player_transform.translation.truncate();

    let mut can_exit = false;
    for (_, transform) in &door_query {
        if player_pos.distance(transform.translation.truncate()) <= DOOR_DISTANCE {
            can_exit = true;
            break;
        }
    }

    if !can_exit { return; }

    despawn_room(&mut commands, &room_entities);
    advance_room(&mut run);
    spawn_room(&mut commands, &art, &mut run);
}

pub fn restart_run(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut run: ResMut<RunState>,
    _persistent: Res<PersistentState>,
    art: Res<GameArt>,
    room_entities: Query<Entity, With<RoomEntity>>,
    door_entities: Query<Entity, With<Door>>,
) {
    if run.phase != Phase::GameOver || !keyboard.just_pressed(KeyCode::KeyR) { return; }

    despawn_room(&mut commands, &room_entities);
    for entity in &door_entities {
        commands.entity(entity).despawn();
    }

    *run = RunState::default();
    spawn_room(&mut commands, &art, &mut run);
}

pub fn meta_progression_save(mut persistent: ResMut<PersistentState>, run: Res<RunState>) {
    if run.phase != Phase::GameOver || !run.is_changed() { return; }

    let earned = run.floor * 10 + run.room;
    persistent.currency += earned;
    persistent.total_runs += 1;
    persistent.best_floor = persistent.best_floor.max(run.floor);
    persistent.save();

    println!("Run ended! Earned {} currency. Total: {}", earned, persistent.currency);
}