use crate::game::ability::FOUND_ABILITIES;
use crate::game::assets::GameArt;
use crate::game::components::*;
use crate::game::progression::room_kind_for;
use crate::game::state::{Phase, RoomKind, RunState};
use crate::game::sword::SWORDS;
use bevy::prelude::*;
use rand::{Rng, SeedableRng, rngs::StdRng};

use super::actors::{shadow_bundle, spawn_enemy_kind};
use super::setup::{spawn_floor, ACTOR_Z};

pub fn spawn_room(commands: &mut Commands, art: &GameArt, run: &mut RunState) {
    let mut rng = StdRng::seed_from_u64(run.room_seed ^ ((run.floor as u64) << 16) ^ run.room as u64);
    run.current_room = room_kind_for(run);
    run.phase = match run.current_room {
        RoomKind::Combat | RoomKind::Boss => Phase::Fighting,
        RoomKind::Rest | RoomKind::Treasure => Phase::RoomCleared,
    };

    spawn_floor(commands, art, run.current_room);

    if run.current_room == RoomKind::Combat || run.current_room == RoomKind::Boss {
        spawn_obstacles(commands, art, run, &mut rng);
    }

    match run.current_room {
        RoomKind::Combat => spawn_combat_room(commands, art, run, &mut rng),
        RoomKind::Boss => spawn_boss_room(commands, art, run),
        RoomKind::Rest => spawn_rest_room(commands, art, run, &mut rng),
        RoomKind::Treasure => spawn_treasure_room(commands, art, run, &mut rng),
    }

    if run.phase == Phase::RoomCleared {
        spawn_door(commands, art);
    }
}

pub fn spawn_door(commands: &mut Commands, art: &GameArt) {
    commands
        .spawn((
            art.image_sprite(&art.orb, Vec2::new(120.0, 150.0), Color::srgb(0.30, 0.85, 1.0)),
            Transform::from_translation(Vec3::new(0.0, -250.0, 1.5)),
            Door,
            RoomEntity,
        ))
        .with_child((
            art.image_sprite(&art.orb, Vec2::new(48.0, 96.0), Color::srgb(0.85, 0.98, 1.0)),
            Transform::from_xyz(0.0, 0.0, 0.5),
        ));
}

pub fn despawn_room(commands: &mut Commands, room_entities: &Query<Entity, With<RoomEntity>>) {
    for entity in room_entities {
        commands.entity(entity).despawn();
    }
}

fn spawn_obstacles(commands: &mut Commands, art: &GameArt, run: &RunState, rng: &mut StdRng) {
    let count = 2 + (run.floor / 2) as usize;
    for _ in 0..count.min(4) {
        let x: f32 = rng.gen_range(-380.0..380.0);
        let y: f32 = rng.gen_range(-180.0..180.0);

        if x.abs() < 80.0 && y.abs() < 80.0 { continue; }
        if y < -200.0 && x.abs() < 60.0 { continue; }

        let radius = rng.gen_range(30.0..50.0);
        let destructible = rng.gen_bool(0.4);
        let color = if destructible {
            Color::srgb(0.55, 0.35, 0.2)
        } else {
            Color::srgb(0.45, 0.42, 0.48)
        };

        commands.spawn((
            art.image_sprite(&art.orb, Vec2::splat(radius * 2.0), color),
            Transform::from_translation(Vec3::new(x, y, ACTOR_Z - 0.5)),
            Obstacle { radius, destructible, hp: if destructible { 2 } else { 999 } },
            RoomEntity,
        ));
    }
}

fn pick_combat_kind(rng: &mut StdRng) -> EnemyKind {
    match rng.gen_range(0..100) {
        0..16 => EnemyKind::Hunter,
        16..28 => EnemyKind::Bruiser,
        28..40 => EnemyKind::Seeker,
        40..52 => EnemyKind::Charger,
        52..62 => EnemyKind::Bomber,
        62..70 => EnemyKind::Caster,
        70..78 => EnemyKind::Splitter,
        78..84 => EnemyKind::Kitten,
        84..90 => EnemyKind::Scratcher,
        90..95 => EnemyKind::Chonker,
        _ => EnemyKind::ShadowCat,
    }
}

fn spawn_combat_room(commands: &mut Commands, art: &GameArt, run: &RunState, rng: &mut StdRng) {
    let slots = 2 + run.floor as usize;
    for _ in 0..slots {
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        let radius = rng.gen_range(120.0..260.0);
        let pos = Vec2::new(angle.cos(), angle.sin()) * radius;
        let kind = pick_combat_kind(rng);

        if kind == EnemyKind::Kitten {
            let count = rng.gen_range(4..7);
            for _ in 0..count {
                let jitter = Vec2::new(rng.gen_range(-50.0..50.0), rng.gen_range(-50.0..50.0));
                spawn_enemy_kind(commands, art, EnemyKind::Kitten, run.floor, pos + jitter, false);
            }
        } else {
            spawn_enemy_kind(commands, art, kind, run.floor, pos, rng.gen_bool(0.12));
        }
    }
}

fn spawn_boss_room(commands: &mut Commands, art: &GameArt, run: &RunState) {
    let boss_type = match run.floor {
        1 => BossType::GoblinKing,
        2 => BossType::Necromancer,
        _ => BossType::Dragon,
    };
    let boss_hp_bonus = match boss_type {
        BossType::GoblinKing => 0,
        BossType::Necromancer => 20,
        BossType::Dragon => 40,
    };
    let hp: i32 = 90 + ((run.floor + 2) * 30) as i32 + boss_hp_bonus;

    let mut entity = commands.spawn((
        art.goblin.sprite(art.goblin.idle.0, 4.5, Color::srgb(1.0, 0.4, 0.35)),
        bevy::sprite::Anchor::CENTER,
        Transform::from_translation(Vec3::new(0.0, 170.0, ACTOR_Z)),
        Enemy {
            kind: EnemyKind::Boss, hp, max_hp: hp,
            damage: 5 + run.floor as i32 / 2,
            speed: 100.0 + run.floor as f32 * 2.0,
            elite: false, splits: false, explodes: false,
            ammo: match boss_type {
                BossType::GoblinKing => 2,
                BossType::Necromancer => 4,
                BossType::Dragon => 3,
            },
            state: EnemyState::Chase,
            state_timer: Timer::from_seconds(0.1, TimerMode::Once),
            action_cd: Timer::from_seconds(0.6, TimerMode::Once),
            charge_dir: Vec2::ZERO, has_telegraph: false,
        },
        CatAnimation::new(art.goblin.idle, art.goblin.walk, Some(art.goblin.attack), 9),
        BaseTint(Color::WHITE),
        BossPhases { boss_type, current_phase: 1 },
        RoomEntity,
    ));
    entity.with_child(shadow_bundle(art, 156.0));
}

fn spawn_rest_room(commands: &mut Commands, art: &GameArt, run: &RunState, rng: &mut StdRng) {
    let heal_count = 2 + (run.floor / 2) as usize;
    for i in 0..heal_count {
        let offset = Vec2::new(rng.gen_range(-220.0..220.0), rng.gen_range(-120.0..160.0));
        spawn_pickup(commands, art, PickupKind::Heal(2), offset, i as f32);
    }
}

fn spawn_treasure_room(commands: &mut Commands, art: &GameArt, run: &RunState, rng: &mut StdRng) {
    let offset = Vec2::new(0.0, 40.0 + run.floor as f32 * 2.0);
    let roll = rng.gen_range(0..100);
    let pickup = if roll < 40 {
        PickupKind::SwordDrop(rng.gen_range(1..SWORDS.len()))
    } else if roll < 72 {
        PickupKind::AbilityDrop(FOUND_ABILITIES[rng.gen_range(0..FOUND_ABILITIES.len())])
    } else {
        match rng.gen_range(0..3) {
            0 => PickupKind::DamageUp(1),
            1 => PickupKind::SpeedUp(20),
            _ => PickupKind::MaxHpUp(2),
        }
    };
    spawn_pickup(commands, art, pickup, offset, 0.0);
}

pub fn spawn_projectile(commands: &mut Commands, art: &GameArt, origin: Vec2, velocity: Vec2, owner: ProjectileOwner, damage: i32) {
    let (color, size) = match owner {
        ProjectileOwner::Player => (Color::srgb(1.0, 0.92, 0.55), 24.0),
        ProjectileOwner::Enemy => (Color::srgb(1.0, 0.45, 0.50), 22.0),
    };
    commands.spawn((
        art.image_sprite(&art.orb, Vec2::splat(size), color),
        Transform::from_translation(Vec3::new(origin.x, origin.y, 3.0)),
        Projectile { owner, damage, velocity, lifetime: Timer::from_seconds(1.6, TimerMode::Once) },
        RoomEntity,
    ));
}

pub fn spawn_sword_reward(commands: &mut Commands, art: &GameArt, index: usize) {
    spawn_pickup(commands, art, PickupKind::SwordDrop(index), Vec2::new(0.0, 70.0), 0.0);
}

pub fn spawn_gem_reward(commands: &mut Commands, art: &GameArt, pos: Vec2, roll: u32) {
    let kind = match roll % 3 {
        0 => PickupKind::DamageUp(1),
        1 => PickupKind::SpeedUp(15),
        _ => PickupKind::Heal(3),
    };
    spawn_pickup(commands, art, kind, pos, 0.0);
}

fn spawn_pickup(commands: &mut Commands, art: &GameArt, kind: PickupKind, offset: Vec2, phase_offset: f32) {
    let (image, color, size) = match kind {
        PickupKind::Heal(_) => (&art.heart, Color::WHITE, 30.0),
        PickupKind::DamageUp(_) => (&art.gem, Color::srgb(1.0, 0.52, 0.42), 30.0),
        PickupKind::SpeedUp(_) => (&art.gem, Color::srgb(0.48, 0.74, 1.0), 30.0),
        PickupKind::MaxHpUp(_) => (&art.gem, Color::srgb(1.0, 0.88, 0.36), 30.0),
        PickupKind::SwordDrop(index) => (&art.swords[SWORDS[index].icon], Color::WHITE, 52.0),
        PickupKind::AbilityDrop(ability) => (&art.orb, ability.color(), 38.0),
    };
    commands.spawn((
        art.image_sprite(image, Vec2::splat(size), color),
        Transform::from_translation(Vec3::new(offset.x, offset.y, 2.0)),
        Pickup { kind },
        Bob { base_y: offset.y, amplitude: 7.0, speed: 2.6, phase: phase_offset * 1.3 },
        RoomEntity,
    ));
}