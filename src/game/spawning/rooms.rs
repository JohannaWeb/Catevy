use crate::game::ability::FOUND_ABILITIES;
use crate::game::assets::GameArt;
use crate::game::components::*;
use crate::game::progression::room_kind_for;
use crate::game::state::{Phase, RoomKind, RunState};
use crate::game::sword::SWORDS;
use bevy::prelude::*;
use rand::{Rng, SeedableRng, rngs::StdRng};

use super::actors::{shadow_bundle, spawn_enemy_kind};
use super::setup::{ACTOR_Z, spawn_floor};

pub fn spawn_room(
    commands: &mut Commands,
    art: &GameArt,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    run: &mut RunState,
) {
    let mut rng =
        StdRng::seed_from_u64(run.room_seed ^ ((run.floor as u64) << 16) ^ run.room as u64);
    run.current_room = room_kind_for(run);
    if run.current_room.is_depth() {
        spawn_depth_room(commands, meshes, materials, run);
        return;
    }

    run.phase = match run.current_room {
        RoomKind::Combat | RoomKind::Boss => Phase::Fighting,
        RoomKind::Rest
        | RoomKind::Treasure
        | RoomKind::Shop
        | RoomKind::Challenge
        | RoomKind::Secret
        | RoomKind::DepthTransition
        | RoomKind::DepthArena
        | RoomKind::DepthBoss => Phase::RoomCleared,
    };

    spawn_floor(commands, art, run.current_room);
    spawn_room_banner(commands, art, run);

    if run.current_room == RoomKind::Combat {
        spawn_obstacles(commands, art, run, &mut rng);
    }

    match run.current_room {
        RoomKind::Combat => spawn_combat_room(commands, art, run, &mut rng),
        RoomKind::Boss => spawn_boss_room(commands, art, run),
        RoomKind::Rest => spawn_rest_room(commands, art, run, &mut rng),
        RoomKind::Treasure => spawn_treasure_room(commands, art, run, &mut rng),
        // New room types - placeholder for now
        RoomKind::Shop | RoomKind::Challenge | RoomKind::Secret => {
            // Placeholder: spawn a rest room for now
            spawn_rest_room(commands, art, run, &mut rng);
        }
        RoomKind::DepthTransition | RoomKind::DepthArena | RoomKind::DepthBoss => {}
    }

    if run.phase == Phase::RoomCleared {
        spawn_door(commands, art);
    }
}

fn spawn_depth_room(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    run: &mut RunState,
) {
    run.phase = match run.current_room {
        RoomKind::DepthTransition => Phase::RoomCleared,
        RoomKind::DepthArena | RoomKind::DepthBoss => Phase::Fighting,
        _ => Phase::Fighting,
    };

    let floor_material = materials.add(Color::srgb(0.22, 0.19, 0.26));
    let wall_material = materials.add(Color::srgb(0.36, 0.28, 0.42));
    let player_material = materials.add(Color::srgb(0.92, 0.92, 1.0));
    let exit_material = materials.add(Color::srgb(0.20, 0.78, 1.0));
    let boss_material = materials.add(match run.current_room {
        RoomKind::DepthBoss => Color::srgb(0.92, 0.18, 0.24),
        _ => Color::srgb(0.65, 0.30, 0.82),
    });

    commands.spawn((
        DirectionalLight {
            shadow_maps_enabled: true,
            illuminance: 6000.0,
            ..default()
        },
        Transform::from_xyz(-4.0, 8.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        RoomEntity,
    ));
    commands.spawn((
        PointLight {
            intensity: 900.0,
            range: 18.0,
            ..default()
        },
        Transform::from_xyz(0.0, 5.5, 1.5),
        RoomEntity,
    ));

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(14.0, 14.0))),
        MeshMaterial3d(floor_material),
        Transform::from_translation(Vec3::ZERO),
        RoomEntity,
    ));

    for (pos, size) in [
        (Vec3::new(0.0, 0.5, -7.0), Vec3::new(14.0, 1.0, 0.35)),
        (Vec3::new(0.0, 0.5, 7.0), Vec3::new(14.0, 1.0, 0.35)),
        (Vec3::new(-7.0, 0.5, 0.0), Vec3::new(0.35, 1.0, 14.0)),
        (Vec3::new(7.0, 0.5, 0.0), Vec3::new(0.35, 1.0, 14.0)),
    ] {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
            MeshMaterial3d(wall_material.clone()),
            Transform::from_translation(pos),
            RoomEntity,
        ));
    }

    let player_pos = Vec3::new(0.0, 0.45, 4.0);
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.75, 0.9, 0.75))),
        MeshMaterial3d(player_material),
        Transform::from_translation(player_pos),
        DepthPlayer {
            facing: Vec3::NEG_Z,
        },
        RoomEntity,
    ));

    let exit_active = run.phase == Phase::RoomCleared;
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.4, 1.0, 0.35))),
        MeshMaterial3d(exit_material),
        Transform::from_translation(Vec3::new(0.0, 0.5, -5.8)),
        if exit_active {
            Visibility::Visible
        } else {
            Visibility::Hidden
        },
        DepthExit {
            active: exit_active,
        },
        RoomEntity,
    ));

    match run.current_room {
        RoomKind::DepthArena => {
            spawn_depth_boss(
                commands,
                meshes,
                boss_material,
                "Depth Warden",
                55,
                2,
                1.35,
                false,
                Vec3::new(0.0, 0.7, -3.0),
            );
        }
        RoomKind::DepthBoss => {
            spawn_depth_boss(
                commands,
                meshes,
                boss_material,
                "Dragon of Depth",
                135,
                3,
                1.1,
                true,
                Vec3::new(0.0, 0.95, -3.3),
            );
        }
        _ => {}
    }
}

fn spawn_room_banner(commands: &mut Commands, art: &GameArt, run: &RunState) {
    let title = match run.current_room {
        RoomKind::Combat => format!("Floor {} - Ambush", run.floor),
        RoomKind::Boss => format!("Boss: {}", boss_room_name(run.floor)),
        RoomKind::Rest => "Safe Room".to_string(),
        RoomKind::Treasure => "Treasure Room".to_string(),
        RoomKind::Shop => "Shop".to_string(),
        RoomKind::Challenge => "Challenge Room".to_string(),
        RoomKind::Secret => "Secret Room".to_string(),
        RoomKind::DepthTransition | RoomKind::DepthArena | RoomKind::DepthBoss => return,
    };

    commands.spawn((
        Text2d::new(title),
        TextFont {
            font: art.font.clone().into(),
            font_size: FontSize::Px(34.0),
            ..default()
        },
        TextColor(Color::srgba(1.0, 0.95, 0.78, 0.95)),
        Transform::from_translation(Vec3::new(0.0, 250.0, 18.0)),
        RoomBanner {
            life: Timer::from_seconds(1.65, TimerMode::Once),
        },
        RoomEntity,
    ));
}

fn boss_room_name(floor: u32) -> &'static str {
    match floor {
        1 => "Goblin King",
        2 => "Necromancer",
        _ => "Dragon",
    }
}

#[allow(clippy::too_many_arguments)]
fn spawn_depth_boss(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: Handle<StandardMaterial>,
    name: &'static str,
    hp: i32,
    damage: i32,
    speed: f32,
    final_boss: bool,
    pos: Vec3,
) {
    let size = if final_boss { 1.55 } else { 1.15 };
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(size, size, size))),
        MeshMaterial3d(material),
        Transform::from_translation(pos),
        DepthBoss {
            name,
            hp,
            max_hp: hp,
            damage,
            speed,
            attack_timer: Timer::from_seconds(if final_boss { 1.4 } else { 1.7 }, TimerMode::Once),
            final_boss,
        },
        RoomEntity,
    ));
}

pub fn spawn_door(commands: &mut Commands, art: &GameArt) {
    commands
        .spawn((
            art.image_sprite(
                &art.orb,
                Vec2::new(120.0, 150.0),
                Color::srgb(0.30, 0.85, 1.0),
            ),
            Transform::from_translation(Vec3::new(0.0, -250.0, 1.5)),
            Door,
            RoomEntity,
        ))
        .with_child((
            art.image_sprite(
                &art.orb,
                Vec2::new(48.0, 96.0),
                Color::srgb(0.85, 0.98, 1.0),
            ),
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

        if x.abs() < 80.0 && y.abs() < 80.0 {
            continue;
        }
        if y < -200.0 && x.abs() < 60.0 {
            continue;
        }

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
            Obstacle {
                radius,
                destructible,
                hp: if destructible { 2 } else { 999 },
            },
            RoomEntity,
        ));
    }
}

/// Themed enemy groups that spawn together for synergistic combat
enum EnemyGroup {
    /// Swarm of small, weak enemies
    Swarm,
    /// Frontline fighters with ranged support
    Balanced,
    /// Heavy hitters with backup
    Assault,
    /// Ranged specialists with protection
    RangedSquad,
    /// Support enemies that buff others
    SupportGroup,
    /// Elite pack - powerful enemies
    Elite,
    /// Necromancer theme - revives and undead
    Undead,
    /// Flying enemies that ignore obstacles
    Flying,
}

/// Enemy spawn patterns organized by floor theme
fn get_floor_enemy_pool(floor: u32) -> Vec<(EnemyKind, f32)> {
    match floor {
        1 => vec![
            (EnemyKind::Hunter, 3.0),      // Common basic enemy
            (EnemyKind::Bruiser, 2.0),     // Tanky melee
            (EnemyKind::Scratcher, 1.5),   // Fast attacker
            (EnemyKind::Kitten, 1.0),      // Swarm enemy
        ],
        2 => vec![
            (EnemyKind::Hunter, 2.5),
            (EnemyKind::Seeker, 2.0),       // Ranged enemy
            (EnemyKind::Charger, 2.0),      // Rushdown enemy
            (EnemyKind::Bomber, 1.5),      // Explosion enemy
            (EnemyKind::Splitter, 1.5),    // Splits into kittens
            (EnemyKind::ShieldBearer, 1.0), // New: frontal shield
        ],
        3 => vec![
            (EnemyKind::Caster, 2.5),      // Projectile spammer
            (EnemyKind::Summoner, 1.5),    // Summons allies
            (EnemyKind::ShadowCat, 1.5),   // Stealth enemy
            (EnemyKind::FlyingCat, 2.0),   // New: ignores obstacles
            (EnemyKind::Bruiser, 2.0),
            (EnemyKind::Charger, 1.5),
        ],
        4 => vec![
            (EnemyKind::Chonker, 2.0),      // Heavy enemy
            (EnemyKind::NecromancerCat, 2.0), // New: revives corpses
            (EnemyKind::Goliath, 1.5),     // New: grows when hit
            (EnemyKind::MimicChest, 1.5),  // New: ambush enemy
            (EnemyKind::Seeker, 2.0),
            (EnemyKind::Bomber, 1.5),
        ],
        _ => vec![
            // Later floors: all enemy types with danger emphasis
            (EnemyKind::NecromancerCat, 2.0),
            (EnemyKind::Goliath, 2.0),
            (EnemyKind::ShadowCat, 1.5),
            (EnemyKind::Chonker, 1.5),
            (EnemyKind::Summoner, 1.5),
            (EnemyKind::MimicChest, 1.0),
            (EnemyKind::FlyingCat, 1.5),
        ],
    }
}

/// Pick an enemy group based on floor and room number
fn pick_enemy_group(floor: u32, room: u32, rng: &mut StdRng) -> EnemyGroup {
    let roll = rng.gen_range(0..100);

    // Higher floors have more elite groups
    let elite_chance = (floor as f32 * 5.0).min(25.0);
    let support_chance = 15.0;
    let ranged_chance = 20.0;

    if roll < elite_chance as u32 && floor >= 3 {
        EnemyGroup::Elite
    } else if roll < (elite_chance + support_chance) as u32 && floor >= 2 {
        EnemyGroup::SupportGroup
    } else if roll < (elite_chance + support_chance + ranged_chance) as u32 && floor >= 2 {
        EnemyGroup::RangedSquad
    } else if roll < (elite_chance + support_chance + ranged_chance + 15.0) as u32 && floor >= 3 {
        EnemyGroup::Undead
    } else if roll < (elite_chance + support_chance + ranged_chance + 25.0) as u32 && floor >= 2 {
        EnemyGroup::Flying
    } else if roll < (elite_chance + support_chance + ranged_chance + 40.0) as u32 {
        EnemyGroup::Assault
    } else if roll < (elite_chance + support_chance + ranged_chance + 60.0) as u32 {
        EnemyGroup::Balanced
    } else {
        EnemyGroup::Swarm
    }
}

/// Spawn a themed group of enemies
fn spawn_enemy_group(
    commands: &mut Commands,
    art: &GameArt,
    run: &RunState,
    rng: &mut StdRng,
    group: EnemyGroup,
    center: Vec2,
) {
    let floor = run.floor;

    match group {
        EnemyGroup::Swarm => {
            // 5-8 weak enemies in a cluster
            let count = rng.gen_range(5..9);
            for i in 0..count {
                let angle = (i as f32 / count as f32) * std::f32::consts::TAU;
                let radius = rng.gen_range(40.0..80.0);
                let offset = Vec2::new(angle.cos(), angle.sin()) * radius;
                let kind = weighted_pick(&get_floor_enemy_pool(floor), rng);
                spawn_enemy_kind(commands, art, kind, floor, center + offset, false);
            }
        }
        EnemyGroup::Balanced => {
            // 2-3 frontline + 1-2 ranged
            let frontline = rng.gen_range(2..4);
            for i in 0..frontline {
                let angle = (i as f32 / frontline as f32) * std::f32::consts::TAU;
                let radius = rng.gen_range(60.0..100.0);
                let offset = Vec2::new(angle.cos(), angle.sin()) * radius;
                let kind = if rng.gen_bool(0.6) {
                    EnemyKind::Hunter
                } else {
                    EnemyKind::Bruiser
                };
                spawn_enemy_kind(commands, art, kind, floor, center + offset, false);
            }

            let ranged = rng.gen_range(1..3);
            for i in 0..ranged {
                let angle = ((i as f32 / ranged as f32) + 0.5) * std::f32::consts::TAU;
                let radius = rng.gen_range(150.0..200.0);
                let offset = Vec2::new(angle.cos(), angle.sin()) * radius;
                let kind = if floor >= 3 && rng.gen_bool(0.5) {
                    EnemyKind::Caster
                } else {
                    EnemyKind::Seeker
                };
                spawn_enemy_kind(commands, art, kind, floor, center + offset, false);
            }
        }
        EnemyGroup::Assault => {
            // Heavy enemies with backup
            let heavies = rng.gen_range(1..3);
            for i in 0..heavies {
                let angle = (i as f32 / heavies as f32) * std::f32::consts::TAU;
                let radius = rng.gen_range(80.0..120.0);
                let offset = Vec2::new(angle.cos(), angle.sin()) * radius;
                let kind = if floor >= 4 && rng.gen_bool(0.4) {
                    EnemyKind::Goliath
                } else {
                    EnemyKind::Chonker
                };
                spawn_enemy_kind(commands, art, kind, floor, center + offset, rng.gen_bool(0.2));
            }

            // Support units
            let support = rng.gen_range(2..4);
            for i in 0..support {
                let angle = (i as f32 / support as f32) * std::f32::consts::TAU + 0.3;
                let radius = rng.gen_range(130.0..180.0);
                let offset = Vec2::new(angle.cos(), angle.sin()) * radius;
                let kind = weighted_pick(&get_floor_enemy_pool(floor), rng);
                spawn_enemy_kind(commands, art, kind, floor, center + offset, false);
            }
        }
        EnemyGroup::RangedSquad => {
            // Multiple ranged enemies positioned around arena
            let ranged_count = rng.gen_range(3..5);
            for i in 0..ranged_count {
                let angle = (i as f32 / ranged_count as f32) * std::f32::consts::TAU;
                let radius = rng.gen_range(180.0..240.0);
                let offset = Vec2::new(angle.cos(), angle.sin()) * radius;
                let kind = if floor >= 3 {
                    if rng.gen_bool(0.5) { EnemyKind::Caster } else { EnemyKind::Seeker }
                } else {
                    EnemyKind::Seeker
                };
                spawn_enemy_kind(commands, art, kind, floor, center + offset, false);
            }

            // Single melee guard
            spawn_enemy_kind(commands, art, EnemyKind::Bruiser, floor, center, false);
        }
        EnemyGroup::SupportGroup => {
            // Summoner/Necromancer with minions
            let support_kind = if floor >= 4 && rng.gen_bool(0.4) {
                EnemyKind::NecromancerCat
            } else {
                EnemyKind::Summoner
            };
            spawn_enemy_kind(commands, art, support_kind, floor, center, rng.gen_bool(0.25));

            // Minions to protect
            let minions = rng.gen_range(3..5);
            for i in 0..minions {
                let angle = (i as f32 / minions as f32) * std::f32::consts::TAU;
                let radius = rng.gen_range(80.0..120.0);
                let offset = Vec2::new(angle.cos(), angle.sin()) * radius;
                spawn_enemy_kind(commands, art, EnemyKind::Hunter, floor, center + offset, false);
            }
        }
        EnemyGroup::Elite => {
            // 2-3 powerful enemies, all elite
            let count = rng.gen_range(2..4);
            for i in 0..count {
                let angle = (i as f32 / count as f32) * std::f32::consts::TAU;
                let radius = rng.gen_range(100.0..150.0);
                let offset = Vec2::new(angle.cos(), angle.sin()) * radius;
                let kind = match floor {
                    1..=2 => if rng.gen_bool(0.5) { EnemyKind::Bruiser } else { EnemyKind::Charger },
                    3 => if rng.gen_bool(0.5) { EnemyKind::Chonker } else { EnemyKind::ShadowCat },
                    _ => if rng.gen_bool(0.5) { EnemyKind::Goliath } else { EnemyKind::NecromancerCat },
                };
                spawn_enemy_kind(commands, art, kind, floor, center + offset, true); // Always elite
            }
        }
        EnemyGroup::Undead => {
            // Necromancer with corpses to revive
            spawn_enemy_kind(commands, art, EnemyKind::NecromancerCat, floor, center + Vec2::new(0.0, 50.0), rng.gen_bool(0.3));

            // Some "corpses" (already-dead enemies that will be revived)
            let corpses = rng.gen_range(2..4);
            for i in 0..corpses {
                let angle = (i as f32 / corpses as f32) * std::f32::consts::TAU;
                let radius = rng.gen_range(60.0..100.0);
                let offset = Vec2::new(angle.cos(), angle.sin()) * radius;
                // These will be targets for revival
                spawn_enemy_kind(commands, art, EnemyKind::Kitten, floor, center + offset, false);
            }
        }
        EnemyGroup::Flying => {
            // Flying enemies that ignore obstacles
            let count = rng.gen_range(3..6);
            for i in 0..count {
                let angle = (i as f32 / count as f32) * std::f32::consts::TAU;
                let radius = rng.gen_range(100.0..200.0);
                let offset = Vec2::new(angle.cos(), angle.sin()) * radius;
                spawn_enemy_kind(commands, art, EnemyKind::FlyingCat, floor, center + offset, false);
            }

            // One ground enemy for variety
            let ground = weighted_pick(&get_floor_enemy_pool(floor), rng);
            spawn_enemy_kind(commands, art, ground, floor, center, false);
        }
    }
}

/// Weighted random pick from enemy pool
fn weighted_pick(pool: &[(EnemyKind, f32)], rng: &mut StdRng) -> EnemyKind {
    let total_weight: f32 = pool.iter().map(|(_, w)| w).sum();
    let mut roll = rng.r#gen::<f32>() * total_weight;

    for (kind, weight) in pool {
        roll -= weight;
        if roll <= 0.0 {
            return *kind;
        }
    }

    pool[0].0 // Fallback
}

fn spawn_combat_room(commands: &mut Commands, art: &GameArt, run: &RunState, rng: &mut StdRng) {
    // Determine number of enemy groups based on floor
    let group_count = match run.floor {
        1 => 1,
        2 => rng.gen_range(1..3),
        3 => rng.gen_range(2..3),
        _ => rng.gen_range(2..4),
    };

    for group_idx in 0..group_count {
        // Position groups around the arena
        let group_angle = (group_idx as f32 / group_count as f32) * std::f32::consts::TAU
            + rng.gen_range(-0.3..0.3);
        let group_radius = rng.gen_range(100.0..220.0);
        let group_center = Vec2::new(group_angle.cos(), group_angle.sin()) * group_radius;

        // Pick a themed group
        let group = pick_enemy_group(run.floor, run.room, rng);
        spawn_enemy_group(commands, art, run, rng, group, group_center);
    }

    // Occasionally add a solo elite enemy on later floors
    if run.floor >= 2 && rng.gen_bool(0.15) {
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        let radius = rng.gen_range(200.0..280.0);
        let pos = Vec2::new(angle.cos(), angle.sin()) * radius;

        let kind = match run.floor {
            2 => EnemyKind::Chonker,
            3 => EnemyKind::ShadowCat,
            _ => EnemyKind::Goliath,
        };
        spawn_enemy_kind(commands, art, kind, run.floor, pos, true);
    }
}

fn spawn_boss_room(commands: &mut Commands, art: &GameArt, run: &RunState) {
    let boss_type = match run.floor {
        1 => BossType::GoblinKing,
        2 => BossType::Necromancer,
        _ => BossType::Dragon,
    };
    let hp: i32 = match boss_type {
        BossType::GoblinKing => 80,
        BossType::Necromancer => 75 + ((run.floor + 2) * 25) as i32 + 20,
        BossType::Dragon => 75 + ((run.floor + 2) * 25) as i32 + 40,
    };
    let damage = match boss_type {
        BossType::GoblinKing => 2,
        _ => 3 + run.floor as i32 / 2,
    };
    let speed = match boss_type {
        BossType::GoblinKing => 78.0,
        _ => 100.0 + run.floor as f32 * 2.0,
    };
    let action_cd = match boss_type {
        BossType::GoblinKing => 0.7,
        _ => 0.6,
    };

    let mut entity = commands.spawn((
        art.goblin
            .sprite(art.goblin.idle.0, 4.5, Color::srgb(1.0, 0.4, 0.35)),
        bevy::sprite::Anchor::CENTER,
        Transform::from_translation(Vec3::new(0.0, 170.0, ACTOR_Z)),
        Enemy {
            kind: EnemyKind::Boss,
            hp,
            max_hp: hp,
            damage,
            speed,
            elite: false,
            splits: false,
            explodes: false,
            ammo: match boss_type {
                BossType::GoblinKing => 2,
                BossType::Necromancer => 4,
                BossType::Dragon => 3,
            },
            state: EnemyState::Chase,
            state_timer: Timer::from_seconds(0.1, TimerMode::Once),
            action_cd: Timer::from_seconds(action_cd, TimerMode::Once),
            charge_dir: Vec2::ZERO,
            has_telegraph: false,
        },
        CatAnimation::new(art.goblin.idle, art.goblin.walk, Some(art.goblin.attack), 9),
        BaseTint(Color::WHITE),
        BossPhases {
            boss_type,
            current_phase: 1,
        },
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

pub fn spawn_projectile(
    commands: &mut Commands,
    art: &GameArt,
    origin: Vec2,
    velocity: Vec2,
    owner: ProjectileOwner,
    damage: i32,
) {
    let (color, size) = match owner {
        ProjectileOwner::Player => (Color::srgb(1.0, 0.92, 0.55), 24.0),
        ProjectileOwner::Enemy => (Color::srgb(1.0, 0.45, 0.50), 22.0),
    };
    commands.spawn((
        art.image_sprite(&art.orb, Vec2::splat(size), color),
        Transform::from_translation(Vec3::new(origin.x, origin.y, 3.0)),
        Projectile {
            owner,
            damage,
            velocity,
            lifetime: Timer::from_seconds(1.6, TimerMode::Once),
        },
        RoomEntity,
    ));
}

pub fn spawn_sword_reward(commands: &mut Commands, art: &GameArt, index: usize) {
    spawn_pickup(
        commands,
        art,
        PickupKind::SwordDrop(index),
        Vec2::new(0.0, 70.0),
        0.0,
    );
}

pub fn spawn_gem_reward(commands: &mut Commands, art: &GameArt, pos: Vec2, roll: u32) {
    let kind = match roll % 3 {
        0 => PickupKind::DamageUp(1),
        1 => PickupKind::SpeedUp(15),
        _ => PickupKind::Heal(3),
    };
    spawn_pickup(commands, art, kind, pos, 0.0);
}

fn spawn_pickup(
    commands: &mut Commands,
    art: &GameArt,
    kind: PickupKind,
    offset: Vec2,
    phase_offset: f32,
) {
    let (image, color, size) = match kind {
        PickupKind::Heal(_) => (&art.heart, Color::WHITE, 30.0),
        PickupKind::DamageUp(_) => (&art.gem, Color::srgb(1.0, 0.52, 0.42), 30.0),
        PickupKind::SpeedUp(_) => (&art.gem, Color::srgb(0.48, 0.74, 1.0), 30.0),
        PickupKind::MaxHpUp(_) => (&art.gem, Color::srgb(1.0, 0.88, 0.36), 30.0),
        PickupKind::SwordDrop(index) => (&art.swords[SWORDS[index].icon], Color::WHITE, 52.0),
        PickupKind::AbilityDrop(ability) => (&art.orb, ability.color(), 38.0),
        PickupKind::Currency(_) => (&art.orb, Color::srgb(1.0, 0.85, 0.0), 35.0), // Gold coin color
    };
    commands.spawn((
        art.image_sprite(image, Vec2::splat(size), color),
        Transform::from_translation(Vec3::new(offset.x, offset.y, 2.0)),
        Pickup { kind },
        Bob {
            base_y: offset.y,
            amplitude: 7.0,
            speed: 2.6,
            phase: phase_offset * 1.3,
        },
        RoomEntity,
    ));
}
