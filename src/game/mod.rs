mod ability;
mod assets;
mod components;
mod progression;
mod spawning;
mod state;
mod sword;
mod systems;

use crate::game::components::{CombatDebug, HitStop};
use crate::game::state::{GameMode, GameState, PersistentState, RunState, ScreenShake};
use crate::game::systems::{LastMouseAim, ComboState};
use bevy::prelude::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        // Load persistent state from disk
        let persistent = PersistentState::load();

        app.insert_resource(RunState::new(&persistent))
            .insert_resource(persistent)
            .init_resource::<ScreenShake>()
            .init_resource::<HitStop>()
            .init_resource::<CombatDebug>()
            .init_resource::<LastMouseAim>()
            .init_resource::<ComboState>()
            .init_state::<GameState>()
            .add_sub_state::<GameMode>()
            // Startup systems
            .add_systems(Startup, assets::setup_assets)
            .add_systems(Startup, spawning::setup_world.after(assets::setup_assets))
            // Main menu systems (run when in MainMenu state)
            .add_systems(
                Update,
                (
                    systems::spawn_main_menu,
                    systems::main_menu_input,
                ).run_if(in_state(GameState::MainMenu)),
            )
            // Gameplay systems (run when in Playing state)
            .add_systems(
                Update,
                (
                    systems::apply_hitstop,
                    systems::player_movement,
                    systems::player_swing,
                    systems::use_abilities,
                    systems::sync_game_mode,
                    systems::dash_flicker,
                    systems::update_slashes,
                    systems::slash_hit_obstacles,
                    systems::projectile_hit_obstacles,
                    systems::enemy_ai,
                    systems::tick_knockback_enemy_timers,
                    systems::enemy_synergies,
                    systems::update_projectiles,
                    systems::resolve_enemy_deaths,
                    systems::update_explosions,
                    systems::collect_pickups,
                    systems::shop_item_interact,
                    systems::process_room_clear.before(systems::door_interact),
                    systems::door_interact,
                    systems::debug_jump_to_depth,
                ).run_if(in_state(GameState::Playing)),
            )
            // Depth prototype systems (run when in Playing state)
            .add_systems(
                Update,
                (
                    systems::depth_player_movement,
                    systems::depth_player_combat,
                    systems::depth_boss_ai,
                    systems::update_depth_projectiles,
                    systems::update_depth_slashes,
                    systems::depth_room_progression.before(systems::depth_exit_interact),
                    systems::depth_exit_interact,
                ).run_if(in_state(GameState::Playing)),
            )
            // Visual effects (run when in Playing or Paused state - for visual continuity)
            .add_systems(
                Update,
                (
                    systems::animate_cats,
                    systems::bob_pickups,
                    systems::tick_hit_flash,
                    systems::update_particles,
                    systems::update_telegraphs,
                    systems::combo_decay,
                    systems::update_combat_debug,
                    systems::update_boss_health_bar,
                    systems::update_hud,
                    systems::update_damage_numbers,
                    systems::update_room_banners,
                    systems::depth_camera_follow,
                    systems::screen_shake,
                    systems::update_low_health_warning,
                    systems::update_afterimages,
                    systems::update_knockback,
                ).run_if(in_state(GameState::Playing)),
            )
            // Pause systems
            .add_systems(
                Update,
                (
                    systems::toggle_pause,
                ).run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                (
                    systems::spawn_pause_menu,
                    systems::pause_menu_input,
                ).run_if(in_state(GameState::Paused)),
            )
            // Game over systems
            .add_systems(
                Update,
                (
                    systems::spawn_game_over_screen,
                    systems::restart_from_game_over,
                    systems::meta_progression_save,
                ).run_if(in_state(GameState::GameOver)),
            )
            // Cleanup transitions
            .add_systems(OnExit(GameState::MainMenu), systems::cleanup_main_menu)
            .add_systems(OnExit(GameState::Paused), systems::cleanup_pause_menu)
            .add_systems(OnExit(GameState::GameOver), systems::cleanup_game_over)
            .add_systems(OnEnter(GameMode::TwoD), systems::enter_2d_mode)
            .add_systems(OnEnter(GameMode::Depth), systems::enter_depth_mode);
    }
}
