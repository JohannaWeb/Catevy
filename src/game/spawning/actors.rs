use crate::game::assets::{CAT_IDLE, CAT_WALK, GameArt, KNIGHT_IDLE, KNIGHT_RUN};
use crate::game::components::*;
use bevy::prelude::*;
use bevy::sprite::Anchor;

use super::setup::{ACTOR_Z, SHADOW_Z};

/// A soft drop shadow placed under a cat actor.
pub fn shadow_bundle(art: &GameArt, size: f32) -> impl Bundle {
    (
        art.image_sprite(&art.shadow, Vec2::new(size * 0.6, size * 0.3), Color::WHITE),
        Transform::from_xyz(0.0, -size * 0.42, SHADOW_Z - ACTOR_Z),
    )
}

pub fn spawn_player(commands: &mut Commands, art: &GameArt) {
    let scale = 3.4;
    commands
        .spawn((
            art.knight_sprite(KNIGHT_IDLE.0, scale, Color::WHITE),
            Anchor::CENTER,
            Transform::from_translation(Vec3::new(0.0, 0.0, ACTOR_Z)),
            Player,
            Facing(Vec2::X),
            CatAnimation::new(KNIGHT_IDLE, KNIGHT_RUN, Some((20, 29)), 14),
            BaseTint(Color::WHITE),
        ))
        .with_child((
            art.image_sprite(&art.shadow, Vec2::new(54.0, 20.0), Color::WHITE),
            Transform::from_xyz(0.0, -40.0, SHADOW_Z - ACTOR_Z),
        ));
}

pub fn spawn_enemy_kind(
    commands: &mut Commands,
    art: &GameArt,
    kind: EnemyKind,
    floor: u32,
    pos: Vec2,
    elite: bool,
) {
    let (mut hp, mut damage, speed, _default_color, mut size) = enemy_stats(kind, floor);
    if elite {
        hp *= 2;
        damage += 1 + floor as i32 / 3;
        size *= 1.35;
    }

    // Use the appropriate color for this enemy type
    let enemy_color = GameArt::enemy_color(kind);

    let (sprite, anim, tint) = if let Some(m) = art.monster_for(kind) {
        let scale = size / m.cell.y;
        let mut a = CatAnimation::new(m.idle, m.walk, Some(m.attack), 9);
        a.moving = true;
        (m.sprite(m.walk.0, scale, enemy_color), a, enemy_color)
    } else {
        let mut a = CatAnimation::new(CAT_IDLE, CAT_WALK, None, 8);
        a.moving = true;
        (art.cat_sprite(CAT_WALK.0, size, enemy_color), a, enemy_color)
    };

    let mut entity = commands.spawn((
        sprite,
        Anchor::CENTER,
        Transform::from_translation(Vec3::new(pos.x, pos.y, ACTOR_Z)),
        Enemy {
            kind,
            hp,
            max_hp: hp,
            damage,
            speed,
            elite,
            splits: kind == EnemyKind::Splitter,
            explodes: kind == EnemyKind::Bomber,
            ammo: match kind {
                EnemyKind::Summoner => 4,
                EnemyKind::Caster => 3,
                EnemyKind::NecromancerCat => 2, // Max revives
                _ => 0,
            },
            state: EnemyState::Chase,
            state_timer: Timer::from_seconds(0.1, TimerMode::Once),
            action_cd: Timer::from_seconds(0.6, TimerMode::Once),
            charge_dir: Vec2::ZERO,
            has_telegraph: false,
        },
        anim,
        BaseTint(tint),
        RoomEntity,
    ));
    entity.with_child(shadow_bundle(art, size));
    if elite {
        entity.with_child((
            art.image_sprite(&art.orb, Vec2::splat(size * 1.5), enemy_color.with_alpha(0.32)),
            Transform::from_xyz(0.0, 0.0, -0.6),
        ));
    }

    // Add enemy-specific components
    use crate::game::components::{
        Flying, Goliath, MimicChest, NecromancerCat, PickupKind, ShieldBearer,
    };
    match kind {
        EnemyKind::NecromancerCat => {
            entity.insert(NecromancerCat {
                revive_range: 120.0,
                revive_timer: Timer::from_seconds(4.0, TimerMode::Once),
                max_revives: 2,
            });
        }
        EnemyKind::ShieldBearer => {
            entity.insert(ShieldBearer {
                shield_angle: 1.2, // ~70 degree cone
            });
        }
        EnemyKind::FlyingCat => {
            entity.insert(Flying);
        }
        EnemyKind::MimicChest => {
            entity.insert(MimicChest {
                disguise_kind: PickupKind::Heal(2), // Disguised as health pickup
                triggered: false,
                trigger_range: 50.0,
            });
        }
        EnemyKind::Goliath => {
            entity.insert(Goliath {
                growth_per_hit: 0.15,
                max_scale: 2.5,
            });
        }
        _ => {}
    }
}

fn enemy_stats(kind: EnemyKind, floor: u32) -> (i32, i32, f32, Color, f32) {
    let f = floor as i32;
    let ff = floor as f32;
    match kind {
        EnemyKind::Hunter => (
            2 + f,
            1 + f / 4,
            130.0 + ff * 6.0,
            Color::srgb(1.0, 0.66, 0.45),
            74.0,
        ),
        EnemyKind::Bruiser => (
            5 + f * 2,
            2 + f / 3,
            92.0 + ff * 4.0,
            Color::srgb(0.86, 0.40, 0.40),
            88.0,
        ),
        EnemyKind::Seeker => (
            3 + f,
            1 + f / 3,
            150.0 + ff * 7.0,
            Color::srgb(0.72, 0.55, 1.0),
            74.0,
        ),
        EnemyKind::Boss => (
            60 + f * 20,
            3 + f / 3,
            100.0 + ff * 2.0,
            Color::srgb(1.0, 0.34, 0.40),
            156.0,
        ),
        EnemyKind::Charger => (
            6 + f * 2,
            3 + f / 2,
            120.0 + ff * 4.0,
            Color::srgb(0.92, 0.66, 0.34),
            100.0,
        ),
        EnemyKind::Bomber => (
            2 + f / 2,
            4 + f / 2,
            150.0 + ff * 8.0,
            Color::srgb(0.95, 0.40, 0.28),
            64.0,
        ),
        EnemyKind::Kitten => (1, 1, 170.0 + ff * 8.0, Color::srgb(0.82, 0.82, 0.88), 46.0),
        EnemyKind::Splitter => (
            4 + f,
            2 + f / 3,
            96.0 + ff * 4.0,
            Color::srgb(0.52, 0.86, 0.52),
            80.0,
        ),
        EnemyKind::Caster => (
            3 + f,
            1 + f / 3,
            120.0 + ff * 5.0,
            Color::srgb(0.50, 0.62, 1.0),
            72.0,
        ),
        EnemyKind::Summoner => (
            5 + f,
            1 + f / 4,
            105.0 + ff * 4.0,
            Color::srgb(0.92, 0.50, 0.92),
            82.0,
        ),
        EnemyKind::Scratcher => (
            2 + f / 2,
            1 + f / 4,
            180.0 + ff * 10.0,
            Color::srgb(0.8, 0.2, 0.25),
            56.0,
        ),
        EnemyKind::Chonker => (
            15 + f * 3,
            2 + f / 3,
            70.0 + ff * 2.0,
            Color::srgb(0.9, 0.7, 0.4),
            110.0,
        ),
        EnemyKind::ShadowCat => (
            4 + f,
            2 + f / 3,
            140.0 + ff * 6.0,
            Color::srgb(0.4, 0.2, 0.6),
            68.0,
        ),
        // New enemy types
        EnemyKind::NecromancerCat => (
            6 + f,
            1 + f / 4,
            100.0 + ff * 3.0,
            Color::srgb(0.6, 0.4, 0.8),
            76.0,
        ),
        EnemyKind::ShieldBearer => (
            8 + f * 2,
            2 + f / 3,
            85.0 + ff * 3.0,
            Color::srgb(0.5, 0.6, 0.7),
            92.0,
        ),
        EnemyKind::FlyingCat => (
            3 + f / 2,
            1 + f / 4,
            160.0 + ff * 7.0,
            Color::srgb(0.7, 0.9, 1.0),
            60.0,
        ),
        EnemyKind::MimicChest => (
            4 + f,
            2 + f / 3,
            130.0 + ff * 5.0,
            Color::srgb(0.85, 0.75, 0.4),
            70.0,
        ),
        EnemyKind::Goliath => (
            12 + f * 3,
            3 + f / 2,
            60.0 + ff * 2.0,
            Color::srgb(0.7, 0.5, 0.6),
            120.0,
        ),
    }
}
