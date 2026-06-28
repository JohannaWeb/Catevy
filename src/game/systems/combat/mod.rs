mod damage;
mod death;
mod melee;
mod ranged;

pub use damage::{
    ENEMY_PROJECTILE_RADIUS, PLAYER_HURT_RADIUS, PLAYER_PROJECTILE_RADIUS, apply_hitstop,
    enemy_hurt_radius, enemy_melee_reach, reset_hurt_invuln, segment_intersects_circle,
    spawn_damage_number_colored, update_combat_debug, update_damage_numbers,
};
pub use death::{resolve_enemy_deaths, update_explosions};
pub use melee::update_slashes;
pub use ranged::update_projectiles;
