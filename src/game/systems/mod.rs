mod player;
mod combat;
mod enemy;
mod effects;
mod room;

// Player systems
pub use player::{player_movement, dash_flicker, use_abilities, player_swing};

// Combat systems
pub use combat::{apply_hitstop, update_damage_numbers, update_slashes, update_projectiles, resolve_enemy_deaths, update_explosions};

// Enemy systems
pub use enemy::{enemy_ai, update_telegraphs, enemy_synergies};

// Effects systems
pub use effects::{update_particles, tick_hit_flash, screen_shake, animate_cats};

// Room systems
pub use room::{process_room_clear, door_interact, restart_run, meta_progression_save, collect_pickups, bob_pickups, update_hud, combo_decay};