mod particles;
mod visuals;

pub use particles::{update_particles, spawn_pickup_pop};
pub use visuals::{tick_hit_flash, screen_shake, animate_cats};