mod combat;
mod depth;
mod effects;
mod enemy;
mod menu;
mod player;
mod room;

// Player systems
pub use player::{LastMouseAim, dash_flicker, player_movement, player_swing, use_abilities};

// Combat systems
pub use combat::{
    apply_hitstop, projectile_hit_obstacles, resolve_enemy_deaths, slash_hit_obstacles,
    update_damage_numbers, update_explosions, update_combat_debug, update_projectiles, update_slashes,
};

// Depth prototype systems
pub use depth::{
    debug_jump_to_depth, depth_boss_ai, depth_camera_follow, depth_exit_interact, depth_player_combat,
    depth_player_movement, depth_room_progression, enter_2d_mode, enter_depth_mode, sync_game_mode,
    update_depth_projectiles, update_depth_slashes,
};

// Enemy systems
pub use enemy::{enemy_ai, enemy_synergies, tick_knockback_enemy_timers, update_telegraphs};

// Effects systems
pub use effects::{animate_cats, screen_shake, tick_hit_flash, update_particles, update_room_banners, update_low_health_warning, update_afterimages, update_knockback};

// Menu systems
pub use menu::{
    spawn_main_menu, main_menu_input, spawn_pause_menu, pause_menu_input, toggle_pause,
    cleanup_pause_menu, spawn_game_over_screen, cleanup_game_over, restart_from_game_over,
    cleanup_main_menu,
};

// Room systems
pub use room::{
    bob_pickups, collect_pickups, combo_decay, door_interact, meta_progression_save,
    process_room_clear, shop_item_interact, update_boss_health_bar, update_hud, ComboState,
};
