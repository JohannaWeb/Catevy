use bevy::prelude::*;
use std::collections::HashSet;

use crate::game::sword::SlashStyle;

/// Marker component for the player entity.
#[derive(Component)]
pub struct Player;

/// The direction an entity is facing.
#[derive(Component)]
pub struct Facing(pub Vec2);

/// A live sword swing: a cone of damage that sweeps for a brief moment.
#[derive(Component)]
pub struct Slash {
    pub damage: i32,
    pub reach: f32,
    pub arc: f32,
    pub knockback: f32,
    pub lifesteal: i32,
    pub origin: Vec2,
    pub dir: Vec2,
    pub life: Timer,
    pub hit: HashSet<Entity>,
    pub slash_style: SlashStyle,
    pub is_crit: bool, // Whether this swing is a critical hit
}