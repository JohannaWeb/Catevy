use crate::game::assets::GameArt;
use crate::game::components::{Door, RoomEntity};
use crate::game::state::{GameState, PersistentState, RunState};
use crate::game::spawning::{despawn_room, spawn_room};
use bevy::prelude::*;

// Marker components for menu entities
#[derive(Component)]
pub struct MainMenuRoot;

#[derive(Component)]
pub struct PauseMenuRoot;

#[derive(Component)]
pub struct GameOverRoot;

#[derive(Component)]
pub struct MenuButton {
    pub action: MenuAction,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    Start,
    Resume,
    Quit,
    Restart,
}

/// Spawn the main menu on app start.
pub fn spawn_main_menu(
    mut commands: Commands,
    art: Res<GameArt>,
    menu_query: Query<Entity, With<MainMenuRoot>>,
) {
    // Don't spawn if already exists
    if !menu_query.is_empty() {
        return;
    }

    let font = art.font.clone();
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.08, 0.06, 0.12).with_alpha(0.95)),
            MainMenuRoot,
        ))
        .with_children(|root| {
            // Title
            root.spawn((
                Text::new("CATEVY"),
                TextFont {
                    font: font.clone().into(),
                    font_size: FontSize::Px(72.0),
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.85, 0.4)),
            ));

            // Subtitle
            root.spawn((
                Text::new("A Cat Roguelike"),
                TextFont {
                    font: font.clone().into(),
                    font_size: FontSize::Px(24.0),
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.65, 0.75)),
                Node {
                    margin: UiRect::bottom(Val::Px(60.0)),
                    ..default()
                },
            ));

            // Start button
            root.spawn(Node {
                width: Val::Px(280.0),
                height: Val::Px(60.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::bottom(Val::Px(20.0)),
                ..default()
            })
            .insert(BackgroundColor(Color::srgb(0.2, 0.18, 0.28)))
            .insert(MenuButton { action: MenuAction::Start })
            .with_children(|button| {
                button.spawn((
                    Text::new("Start Game"),
                    TextFont {
                        font: font.clone().into(),
                        font_size: FontSize::Px(26.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.88, 0.95)),
                ));
            });
        });
}

/// Handle main menu button interactions.
pub fn main_menu_input(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut run: ResMut<RunState>,
    persistent: Res<PersistentState>,
    art: Res<GameArt>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    menu_roots: Query<Entity, With<MainMenuRoot>>,
    interaction_query: Query<(&Interaction, &MenuButton), Changed<Interaction>>,
    room_entities: Query<Entity, With<RoomEntity>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    // Check for Enter key or button click
    let mut should_start = false;

    for (interaction, button) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match button.action {
                MenuAction::Start => should_start = true,
                MenuAction::Quit => {
                    // Can't quit in WASM, so just ignore
                    #[cfg(not(target_arch = "wasm32"))]
                    std::process::exit(0);
                }
                _ => {}
            }
        }
    }

    // Also allow Enter key to start
    if keyboard.just_pressed(KeyCode::Enter) {
        should_start = true;
    }

    if should_start {
        // Despawn menu
        for entity in &menu_roots {
            commands.entity(entity).despawn();
        }

        // Reset run state and spawn first room
        *run = RunState::new(&persistent);
        despawn_room(&mut commands, &room_entities);
        spawn_room(&mut commands, &art, &mut meshes, &mut materials, &mut run);

        // Transition to playing
        next_state.set(GameState::Playing);
    }
}

/// Spawn the pause menu when game is paused.
pub fn spawn_pause_menu(
    mut commands: Commands,
    art: Res<GameArt>,
    menu_query: Query<Entity, With<PauseMenuRoot>>,
    game_state: Res<State<GameState>>,
) {
    if *game_state.get() != GameState::Paused {
        return;
    }

    // Don't spawn if already exists
    if !menu_query.is_empty() {
        return;
    }

    let font = art.font.clone();
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.05, 0.04, 0.08).with_alpha(0.85)),
            PauseMenuRoot,
        ))
        .with_children(|root| {
            // Title
            root.spawn((
                Text::new("PAUSED"),
                TextFont {
                    font: font.clone().into(),
                    font_size: FontSize::Px(48.0),
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.9, 0.5)),
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
            ));

            // Resume button
            root.spawn(Node {
                width: Val::Px(280.0),
                height: Val::Px(60.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::bottom(Val::Px(15.0)),
                ..default()
            })
            .insert(BackgroundColor(Color::srgb(0.2, 0.18, 0.28)))
            .insert(MenuButton { action: MenuAction::Resume })
            .with_children(|button| {
                button.spawn((
                    Text::new("Resume"),
                    TextFont {
                        font: font.clone().into(),
                        font_size: FontSize::Px(26.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.88, 0.95)),
                ));
            });

            // Quit to menu button
            root.spawn(Node {
                width: Val::Px(280.0),
                height: Val::Px(60.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            })
            .insert(BackgroundColor(Color::srgb(0.2, 0.18, 0.28)))
            .insert(MenuButton { action: MenuAction::Quit })
            .with_children(|button| {
                button.spawn((
                    Text::new("Quit to Menu"),
                    TextFont {
                        font: font.clone().into(),
                        font_size: FontSize::Px(26.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.88, 0.95)),
                ));
            });
        });
}

/// Handle pause menu button interactions.
pub fn pause_menu_input(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    menu_roots: Query<Entity, With<PauseMenuRoot>>,
    interaction_query: Query<(&Interaction, &MenuButton), Changed<Interaction>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    // Check for ESC key or button click
    let mut should_resume = false;
    let mut should_quit = false;

    for (interaction, button) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match button.action {
                MenuAction::Resume => should_resume = true,
                MenuAction::Quit => should_quit = true,
                _ => {}
            }
        }
    }

    // ESC to resume
    if keyboard.just_pressed(KeyCode::Escape) {
        should_resume = true;
    }

    if should_resume {
        // Despawn pause menu
        for entity in &menu_roots {
            commands.entity(entity).despawn();
        }
        next_state.set(GameState::Playing);
    }

    if should_quit {
        // Despawn pause menu and room entities, return to main menu
        for entity in &menu_roots {
            commands.entity(entity).despawn();
        }
        // Note: Room entities will be despawned when starting a new game
        next_state.set(GameState::MainMenu);
    }
}

/// Toggle pause state when ESC is pressed during gameplay.
pub fn toggle_pause(
    keyboard: Res<ButtonInput<KeyCode>>,
    game_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        match game_state.get() {
            GameState::Playing => {
                next_state.set(GameState::Paused);
            }
            GameState::Paused => {
                next_state.set(GameState::Playing);
            }
            _ => {}
        }
    }
}

/// Clean up pause menu when transitioning back to playing.
pub fn cleanup_pause_menu(
    mut commands: Commands,
    menu_roots: Query<Entity, With<PauseMenuRoot>>,
) {
    for entity in &menu_roots {
        commands.entity(entity).despawn();
    }
}

/// Spawn the game over screen.
pub fn spawn_game_over_screen(
    mut commands: Commands,
    art: Res<GameArt>,
    run: Res<RunState>,
    persistent: Res<crate::game::state::PersistentState>,
    game_state: Res<State<GameState>>,
    game_over_query: Query<Entity, With<GameOverRoot>>,
) {
    if *game_state.get() != GameState::GameOver {
        return;
    }

    // Don't spawn if already exists
    if !game_over_query.is_empty() {
        return;
    }

    let earned = run.floor * 10 + run.room;
    let font = art.font.clone();

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.04, 0.06).with_alpha(0.9)),
            GameOverRoot,
        ))
        .with_children(|root| {
            // Title
            root.spawn((
                Text::new("GAME OVER"),
                TextFont {
                    font: font.clone().into(),
                    font_size: FontSize::Px(56.0),
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.35, 0.35)),
            ));

            // Stats
            root.spawn((
                Text::new(format!("Floor: {}  |  Room: {}", run.floor, run.room)),
                TextFont {
                    font: font.clone().into(),
                    font_size: FontSize::Px(24.0),
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.75, 0.85)),
                Node {
                    margin: UiRect::top(Val::Px(30.0)),
                    ..default()
                },
            ));

            root.spawn((
                Text::new(format!(
                    "Currency Earned: {}  |  Total: {}",
                    earned, persistent.currency
                )),
                TextFont {
                    font: font.clone().into(),
                    font_size: FontSize::Px(20.0),
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.65, 0.75)),
                Node {
                    margin: UiRect::top(Val::Px(10.0)),
                    ..default()
                },
            ));

            // Restart hint
            root.spawn((
                Text::new("Press R to Restart"),
                TextFont {
                    font: font.clone().into(),
                    font_size: FontSize::Px(22.0),
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.55, 0.65)),
                Node {
                    margin: UiRect::top(Val::Px(40.0)),
                    ..default()
                },
            ));
        });
}

/// Clean up game over screen when restarting.
pub fn cleanup_game_over(
    mut commands: Commands,
    game_over_roots: Query<Entity, With<GameOverRoot>>,
) {
    for entity in &game_over_roots {
        commands.entity(entity).despawn();
    }
}

/// Handle restart from game over screen.
pub fn restart_from_game_over(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut run: ResMut<RunState>,
    persistent: Res<PersistentState>,
    art: Res<GameArt>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    room_entities: Query<Entity, With<RoomEntity>>,
    door_entities: Query<Entity, With<Door>>,
    game_over_roots: Query<Entity, With<GameOverRoot>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::KeyR) {
        // Clean up game over screen
        for entity in &game_over_roots {
            commands.entity(entity).despawn();
        }

        // Despawn room
        despawn_room(&mut commands, &room_entities);
        for entity in &door_entities {
            commands.entity(entity).despawn();
        }

        // Reset and start
        *run = RunState::new(&persistent);
        spawn_room(&mut commands, &art, &mut meshes, &mut materials, &mut run);
        next_state.set(GameState::Playing);
    }
}

/// Clean up main menu when transitioning away.
pub fn cleanup_main_menu(
    mut commands: Commands,
    menu_roots: Query<Entity, With<MainMenuRoot>>,
) {
    for entity in &menu_roots {
        commands.entity(entity).despawn();
    }
}