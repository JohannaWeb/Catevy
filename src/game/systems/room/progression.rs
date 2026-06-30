use crate::game::assets::GameArt;
use crate::game::components::*;
use crate::game::progression::advance_room;
use crate::game::spawning::{
    despawn_room, spawn_door, spawn_gem_reward, spawn_room, spawn_sword_reward,
};
use crate::game::state::{GameState, PersistentState, Phase, RoomKind, RunState};
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
    if run.current_room.is_depth() || run.phase != Phase::Fighting || !enemies.is_empty() {
        return;
    }

    run.phase = Phase::RoomCleared;
    if existing_door.is_empty() {
        spawn_door(&mut commands, &art);
        if run.current_room == RoomKind::Boss {
            let index = 1 + (run.floor as usize) % (SWORDS.len() - 1);
            spawn_sword_reward(&mut commands, &art, index);
            spawn_gem_reward(
                &mut commands,
                &art,
                Vec2::new(-120.0, 55.0),
                run.floor * 3,
            );
            spawn_gem_reward(
                &mut commands,
                &art,
                Vec2::new(120.0, 55.0),
                run.floor * 3 + 2,
            );
        }
    }
}

pub fn door_interact(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut run: ResMut<RunState>,
    art: Res<GameArt>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    door_query: Query<(Entity, &Transform), With<Door>>,
    room_entities: Query<Entity, With<RoomEntity>>,
) {
    if run.current_room.is_depth()
        || run.phase != Phase::RoomCleared
        || !keyboard.just_pressed(KeyCode::KeyE)
    {
        return;
    }

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
    spawn_room(&mut commands, &art, &mut meshes, &mut materials, &mut run);
}

pub fn meta_progression_save(
    game_state: Res<State<GameState>>,
    mut persistent: ResMut<PersistentState>,
    run: Res<RunState>,
) {
    if *game_state.get() != GameState::GameOver || !run.is_changed() { return; }

    let earned = run.floor * 10 + run.room;
    persistent.currency += earned;
    persistent.total_runs += 1;
    persistent.best_floor = persistent.best_floor.max(run.floor);
    persistent.save();

    println!("Run ended! Earned {} currency. Total: {}", earned, persistent.currency);
}
