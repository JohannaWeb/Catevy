mod effects;
mod enemy;
mod player;
mod projectiles;

// Re-export all public types
pub use effects::*;
pub use enemy::*;
pub use player::*;
pub use projectiles::*;

use bevy::prelude::*;
use std::time::Duration;

/// Exit door marker.
#[derive(Component)]
pub struct Door;

/// Marks entities that should be despawned when leaving a room.
#[derive(Component)]
pub struct RoomEntity;

/// HUD text marker.
#[derive(Component)]
pub struct HudText;

/// Persistent 2D camera marker.
#[derive(Component)]
pub struct Main2dCamera;

/// Persistent 3D camera marker used by the depth prototype.
#[derive(Component)]
pub struct DepthCamera;

/// Root node for the boss health bar UI.
#[derive(Component)]
pub struct BossHealthBarRoot;

/// Fill node for the boss health bar UI.
#[derive(Component)]
pub struct BossHealthBarFill;

/// Label text for the boss health bar UI.
#[derive(Component)]
pub struct BossHealthBarText;

/// 3D player avatar used after the world unfolds.
#[derive(Component)]
pub struct DepthPlayer {
    pub facing: Vec3,
}

/// 3D room boss/enemy prototype.
#[derive(Component)]
pub struct DepthBoss {
    pub name: &'static str,
    pub hp: i32,
    pub max_hp: i32,
    pub damage: i32,
    pub speed: f32,
    pub attack_timer: Timer,
    pub final_boss: bool,
}

/// 3D projectile travelling on the floor plane.
#[derive(Component)]
pub struct DepthProjectile {
    pub velocity: Vec3,
    pub damage: i32,
    pub lifetime: Timer,
}

/// Short 3D slash visual.
#[derive(Component)]
pub struct DepthSlash {
    pub life: Timer,
}

/// Exit marker for 3D rooms.
#[derive(Component)]
pub struct DepthExit {
    pub active: bool,
}

/// An obstacle that blocks movement and projectiles.
#[derive(Component)]
pub struct Obstacle {
    pub radius: f32,
    pub destructible: bool,
    pub hp: i32,
}

/// Marker for obstacles that can be destroyed.
#[derive(Component)]
pub struct DestructibleObstacle;

/// Shop room state with purchasable items.
#[derive(Component)]
pub struct ShopRoom {
    pub items: Vec<ShopItem>,
}

/// Marker for a shop item entity that can be purchased.
#[derive(Component)]
pub struct ShopItemMarker {
    pub kind: ShopItemKind,
    pub cost: u32,
}

#[derive(Clone)]
pub struct ShopItem {
    pub kind: ShopItemKind,
    pub cost: u32,
    pub purchased: bool,
}

#[derive(Clone)]
pub enum ShopItemKind {
    HealFull,
    DamageUp(i32),
    SpeedUp(i32),
    MaxHpUp(i32),
    Sword(usize),
    Ability(crate::game::ability::Ability),
    Reroll,
}

/// Cached 3D mesh/material handles for depth-room combat spawns.
/// Created once per depth room entry so projectile and slash spawns
/// share handles instead of allocating new GPU assets each frame.
#[derive(Resource)]
pub struct DepthCombatAssets {
    pub slash_mesh: Handle<Mesh>,
    pub slash_material: Handle<StandardMaterial>,
    pub projectile_mesh: Handle<Mesh>,
    pub projectile_material: Handle<StandardMaterial>,
}

/// Drives sprite animation between idle/walk/attack clips.
#[derive(Component)]
pub struct CatAnimation {
    pub idle: (usize, usize),
    pub walk: (usize, usize),
    pub attack: Option<(usize, usize)>,
    pub moving: bool,
    pub attacking: bool,
    pub frame_timer: Timer,
}

impl CatAnimation {
    pub fn new(
        idle: (usize, usize),
        walk: (usize, usize),
        attack: Option<(usize, usize)>,
        fps: u8,
    ) -> Self {
        Self {
            idle,
            walk,
            attack,
            moving: false,
            attacking: false,
            frame_timer: Timer::new(
                Duration::from_secs_f32(1.0 / fps as f32),
                TimerMode::Repeating,
            ),
        }
    }

    pub fn clip(&self) -> (usize, usize) {
        if self.attacking {
            if let Some(attack) = self.attack {
                return attack;
            }
        }
        if self.moving { self.walk } else { self.idle }
    }

    pub fn start_attack(&mut self) {
        if self.attack.is_some() {
            self.attacking = true;
            self.frame_timer.reset();
        }
    }
}
