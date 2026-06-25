use bevy::prelude::*;
use crate::game::ability::Ability;

/// A projectile that moves and damages entities.
#[derive(Component)]
pub struct Projectile {
    pub owner: ProjectileOwner,
    pub damage: i32,
    pub velocity: Vec2,
    pub lifetime: Timer,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ProjectileOwner {
    Player,
    Enemy,
}

/// A pickup item that the player can collect.
#[derive(Component)]
pub struct Pickup {
    pub kind: PickupKind,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PickupKind {
    Heal(i32),
    DamageUp(i32),
    SpeedUp(i32),
    MaxHpUp(i32),
    SwordDrop(usize),
    AbilityDrop(Ability),
}

/// An expanding blast (purely visual; damage applied when spawned).
#[derive(Component)]
pub struct Explosion {
    pub life: Timer,
    pub max_scale: f32,
}