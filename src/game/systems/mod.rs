mod combat;
mod depth;
mod effects;
mod enemy;
mod player;
mod room;

// Player systems
pub use player::{LastMouseAim, dash_flicker, player_movement, player_swing, use_abilities};

// Combat systems
pub use combat::{
    apply_hitstop, resolve_enemy_deaths, update_damage_numbers, update_explosions,
    update_combat_debug, update_projectiles, update_slashes,
};

// Depth prototype systems
pub use depth::{
    debug_jump_to_depth, depth_boss_ai, depth_camera_follow, depth_exit_interact, depth_player_combat,
    depth_player_movement, depth_room_progression, sync_dimension_view, update_depth_projectiles,
    update_depth_slashes,
};

// Enemy systems
pub use enemy::{enemy_ai, enemy_synergies, update_telegraphs};

// Effects systems
pub use effects::{animate_cats, screen_shake, tick_hit_flash, update_particles, update_room_banners, update_low_health_warning, update_afterimages, update_knockback};

// Room systems
pub use room::{
    bob_pickups, collect_pickups, combo_decay, door_interact, meta_progression_save,
    process_room_clear, restart_run, update_boss_health_bar, update_hud, ComboState,
};
