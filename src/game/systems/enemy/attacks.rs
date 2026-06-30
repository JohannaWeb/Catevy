use crate::game::assets::{GameArt, Sfx};
use crate::game::components::*;
use crate::game::spawning::{spawn_enemy_kind, spawn_projectile};
use crate::game::state::ScreenShake;
use bevy::prelude::*;

use crate::game::systems::combat::{PLAYER_HURT_RADIUS, enemy_melee_reach};
use crate::game::systems::player::rotate;

#[allow(clippy::too_many_arguments)]
pub fn perform_attack(
    commands: &mut Commands,
    art: &GameArt,
    enemy: &mut Enemy,
    pos: Vec2,
    player_pos: Vec2,
    distance: f32,
    dir: Vec2,
    player_hp: &mut i32,
    shake: &mut ScreenShake,
    entity: Entity,
    floor: u32,
    invuln: bool,
    sfx: &Sfx,
    boss_type: Option<BossType>,
) {
    match enemy.kind {
        EnemyKind::Hunter | EnemyKind::Bruiser | EnemyKind::Kitten | EnemyKind::Splitter => {
            if distance <= enemy_melee_reach(enemy.kind) + PLAYER_HURT_RADIUS && !invuln {
                *player_hp -= enemy.damage;
                shake.trauma = (shake.trauma + 0.35).min(1.0);
            }
        }
        EnemyKind::Charger => {}
        EnemyKind::Bomber => {
            explode(
                commands,
                art,
                pos,
                enemy.damage + 2,
                120.0,
                player_pos,
                player_hp,
                shake,
                invuln,
            );
            play(commands, &sfx.explosion, 0.6);
            commands.entity(entity).despawn();
        }
        EnemyKind::Seeker => {
            spawn_projectile(
                commands,
                art,
                pos + dir * 24.0,
                dir * 250.0,
                ProjectileOwner::Enemy,
                enemy.damage,
            );
        }
        EnemyKind::Boss => {
            let hp_percent = enemy.hp as f32 / enemy.max_hp as f32;
            if matches!(boss_type, Some(BossType::GoblinKing)) {
                if hp_percent > 0.66 {
                    spawn_projectile(
                        commands,
                        art,
                        pos + dir * 28.0,
                        dir * 190.0,
                        ProjectileOwner::Enemy,
                        enemy.damage,
                    );
                } else if hp_percent > 0.33 {
                    for a in [-0.18_f32, 0.18] {
                        let d = rotate(dir, a);
                        spawn_projectile(
                            commands,
                            art,
                            pos + d * 28.0,
                            d * 205.0,
                            ProjectileOwner::Enemy,
                            enemy.damage,
                        );
                    }
                } else {
                    for a in [-0.25_f32, 0.0, 0.25] {
                        let d = rotate(dir, a);
                        spawn_projectile(
                            commands,
                            art,
                            pos + d * 28.0,
                            d * 220.0,
                            ProjectileOwner::Enemy,
                            enemy.damage,
                        );
                    }
                }
            } else if matches!(boss_type, Some(BossType::Necromancer)) {
                // Phase 1: summon wave of minions
                // Phase 2: summon one minion + fire projectiles
                // Phase 3: full projectile spread (no more summons)
                if hp_percent > 0.66 {
                    if enemy.ammo > 0 {
                        enemy.ammo -= 1;
                        for k in 0..2 {
                            let side = if k == 0 { 1.0 } else { -1.0 };
                            spawn_enemy_kind(
                                commands, art,
                                EnemyKind::Kitten, floor,
                                pos + Vec2::new(side * 55.0, 25.0),
                                false,
                            );
                        }
                    }
                    spawn_projectile(
                        commands, art,
                        pos + dir * 28.0, dir * 180.0,
                        ProjectileOwner::Enemy, enemy.damage,
                    );
                } else if hp_percent > 0.33 {
                    if enemy.ammo > 0 {
                        enemy.ammo -= 1;
                        spawn_enemy_kind(
                            commands, art,
                            EnemyKind::Scratcher, floor,
                            pos + dir * 50.0,
                            false,
                        );
                    }
                    for a in [-0.2_f32, 0.2] {
                        let d = rotate(dir, a);
                        spawn_projectile(
                            commands, art,
                            pos + d * 28.0, d * 230.0,
                            ProjectileOwner::Enemy, enemy.damage,
                        );
                    }
                } else {
                    for a in [-0.3_f32, 0.0, 0.3] {
                        let d = rotate(dir, a);
                        spawn_projectile(
                            commands, art,
                            pos + d * 28.0, d * 265.0,
                            ProjectileOwner::Enemy, enemy.damage + 1,
                        );
                    }
                }
            } else {
                // Dragon and any future bosses: escalating projectile spread
                if hp_percent > 0.66 {
                    for a in [-0.2_f32, 0.2] {
                        let d = rotate(dir, a);
                        spawn_projectile(
                            commands, art,
                            pos + d * 28.0, d * 250.0,
                            ProjectileOwner::Enemy, enemy.damage,
                        );
                    }
                } else if hp_percent > 0.33 {
                    for a in [-0.3_f32, 0.0, 0.3] {
                        let d = rotate(dir, a);
                        spawn_projectile(
                            commands, art,
                            pos + d * 28.0, d * 275.0,
                            ProjectileOwner::Enemy, enemy.damage,
                        );
                    }
                } else {
                    for a in [-0.4_f32, -0.2, 0.0, 0.2, 0.4] {
                        let d = rotate(dir, a);
                        spawn_projectile(
                            commands, art,
                            pos + d * 28.0, d * 300.0,
                            ProjectileOwner::Enemy, enemy.damage + 1,
                        );
                    }
                }
            }
        }
        EnemyKind::Caster => {
            let n = enemy.ammo.max(1);
            for i in 0..n {
                let a = (i as f32 - (n as f32 - 1.0) / 2.0) * 0.22;
                let d = rotate(dir, a);
                spawn_projectile(
                    commands,
                    art,
                    pos + d * 22.0,
                    d * 230.0,
                    ProjectileOwner::Enemy,
                    enemy.damage,
                );
            }
        }
        EnemyKind::Summoner => {
            if enemy.ammo > 0 {
                enemy.ammo -= 1;
                for k in 0..2 {
                    let side = if k == 0 { 1.0 } else { -1.0 };
                    let spot = pos + Vec2::new(side * 40.0, 30.0);
                    spawn_enemy_kind(commands, art, EnemyKind::Kitten, floor, spot, false);
                }
            }
        }
        EnemyKind::Scratcher => {
            if distance <= enemy_melee_reach(enemy.kind) + PLAYER_HURT_RADIUS && !invuln {
                *player_hp -= enemy.damage;
            }
        }
        EnemyKind::Chonker => {
            enemy.state = EnemyState::Charge;
            enemy.charge_dir = dir;
            enemy.state_timer = Timer::from_seconds(0.6, TimerMode::Once);
        }
        EnemyKind::ShadowCat => {
            if distance <= enemy_melee_reach(enemy.kind) + PLAYER_HURT_RADIUS && !invuln {
                *player_hp -= enemy.damage;
                shake.trauma = (shake.trauma + 0.35).min(1.0);
            }
        }
        // New enemy types - default melee attacks for now
        EnemyKind::NecromancerCat
        | EnemyKind::ShieldBearer
        | EnemyKind::FlyingCat
        | EnemyKind::MimicChest
        | EnemyKind::Goliath => {
            if distance <= enemy_melee_reach(enemy.kind) + PLAYER_HURT_RADIUS && !invuln {
                *player_hp -= enemy.damage;
                shake.trauma = (shake.trauma + 0.35).min(1.0);
            }
        }
    }
}

fn play(commands: &mut Commands, clip: &Handle<bevy::audio::AudioSource>, volume: f32) {
    commands.spawn((
        bevy::audio::AudioPlayer::new(clip.clone()),
        bevy::audio::PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::Linear(volume)),
    ));
}

#[allow(clippy::too_many_arguments)]
fn explode(
    commands: &mut Commands,
    art: &GameArt,
    pos: Vec2,
    damage: i32,
    radius: f32,
    player_pos: Vec2,
    player_hp: &mut i32,
    shake: &mut ScreenShake,
    invuln: bool,
) {
    if !invuln && player_pos.distance(pos) <= radius {
        *player_hp -= damage;
        shake.trauma = (shake.trauma + 0.6).min(1.0);
    }
    commands.spawn((
        art.image_sprite(
            &art.orb,
            Vec2::splat(radius * 2.0),
            Color::srgb(1.0, 0.6, 0.3),
        ),
        Transform::from_translation(pos.extend(3.6)).with_scale(Vec3::splat(0.25)),
        Explosion {
            life: Timer::from_seconds(0.3, TimerMode::Once),
            max_scale: 1.0,
        },
        RoomEntity,
    ));
}
