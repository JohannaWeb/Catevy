mod ability;
mod assets;
mod components;
mod progression;
mod spawning;
mod state;
mod sword;
mod systems;

use crate::game::components::HitStop;
use crate::game::state::{PersistentState, RunState, ScreenShake};
use bevy::prelude::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        // Load persistent state from disk
        let persistent = PersistentState::load();

        app.insert_resource(persistent)
            .init_resource::<RunState>()
            .init_resource::<ScreenShake>()
            .init_resource::<HitStop>()
            .add_systems(Startup, assets::setup_assets)
            .add_systems(Startup, spawning::setup_world.after(assets::setup_assets))
            .add_systems(
                Update,
                (
                    systems::apply_hitstop,
                    systems::player_movement,
                    systems::player_swing,
                    systems::use_abilities,
                    systems::dash_flicker,
                    systems::update_slashes,
                    systems::enemy_ai,
                    systems::enemy_synergies,
                    systems::update_projectiles,
                    systems::resolve_enemy_deaths,
                    systems::update_explosions,
                    systems::collect_pickups,
                    systems::process_room_clear,
                    systems::door_interact,
                    systems::restart_run,
                    systems::meta_progression_save,
                ),
            )
            .add_systems(
                Update,
                (
                    systems::animate_cats,
                    systems::bob_pickups,
                    systems::tick_hit_flash,
                    systems::update_particles,
                    systems::update_telegraphs,
                    systems::combo_decay,
                    systems::update_hud,
                    systems::update_damage_numbers,
                    systems::screen_shake,
                ),
            );
    }
}
