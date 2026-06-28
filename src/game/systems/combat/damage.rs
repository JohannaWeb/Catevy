use crate::game::assets::GameArt;
use crate::game::components::*;
use crate::game::state::RunState;
use bevy::prelude::*;
use bevy::time::{Real, Virtual};

pub const PLAYER_HURT_RADIUS: f32 = 13.0;
pub const ENEMY_PROJECTILE_RADIUS: f32 = 7.0;
pub const PLAYER_PROJECTILE_RADIUS: f32 = 12.0;
pub const HURT_INVULN_SECONDS: f32 = 0.9;

pub fn reset_hurt_invuln(invuln: &mut Timer) {
    *invuln = Timer::from_seconds(HURT_INVULN_SECONDS, TimerMode::Once);
}

pub fn enemy_hurt_radius(kind: EnemyKind, elite: bool) -> f32 {
    let base = match kind {
        EnemyKind::Boss => 58.0,
        EnemyKind::Chonker | EnemyKind::Goliath => 43.0,
        EnemyKind::Charger => 36.0,
        EnemyKind::Bruiser | EnemyKind::ShieldBearer => 34.0,
        EnemyKind::Hunter | EnemyKind::Seeker | EnemyKind::Splitter | EnemyKind::Summoner => 29.0,
        EnemyKind::Caster | EnemyKind::NecromancerCat | EnemyKind::MimicChest => 28.0,
        EnemyKind::Bomber | EnemyKind::ShadowCat | EnemyKind::FlyingCat => 25.0,
        EnemyKind::Scratcher | EnemyKind::Kitten => 22.0,
    };

    if elite { base * 1.25 } else { base }
}

pub fn enemy_melee_reach(kind: EnemyKind) -> f32 {
    match kind {
        EnemyKind::Boss => 50.0,
        EnemyKind::Bruiser | EnemyKind::Goliath | EnemyKind::ShieldBearer => 42.0,
        EnemyKind::Kitten | EnemyKind::Scratcher | EnemyKind::FlyingCat => 34.0,
        _ => 40.0,
    }
}

pub fn segment_intersects_circle(start: Vec2, end: Vec2, center: Vec2, radius: f32) -> bool {
    let segment = end - start;
    let len_sq = segment.length_squared();
    if len_sq <= f32::EPSILON {
        return end.distance_squared(center) <= radius * radius;
    }

    let t = ((center - start).dot(segment) / len_sq).clamp(0.0, 1.0);
    let closest = start + segment * t;
    closest.distance_squared(center) <= radius * radius
}

/// Freezes virtual time while a hit-stop is active, counting down in real time.
pub fn apply_hitstop(
    real: Res<Time<Real>>,
    mut hitstop: ResMut<HitStop>,
    mut virtual_time: ResMut<Time<Virtual>>,
) {
    if hitstop.0 > 0.0 {
        hitstop.0 -= real.delta_secs();
        virtual_time.set_relative_speed(0.0);
    } else {
        virtual_time.set_relative_speed(1.0);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn update_combat_debug(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut debug: ResMut<CombatDebug>,
    run: Res<RunState>,
    art: Res<GameArt>,
    mut commands: Commands,
    visuals: Query<Entity, With<CombatDebugVisual>>,
    player: Query<&Transform, With<Player>>,
    enemies: Query<(&Transform, &Enemy)>,
    projectiles: Query<(&Transform, &Projectile)>,
    slashes: Query<&Slash>,
    obstacles: Query<(&Transform, &Obstacle)>,
) {
    if keyboard.just_pressed(KeyCode::F2) {
        debug.enabled = !debug.enabled;
    }

    for entity in &visuals {
        commands.entity(entity).despawn();
    }

    if !debug.enabled || run.current_room.is_depth() {
        return;
    }

    if let Ok(transform) = player.single() {
        spawn_debug_circle(
            &mut commands,
            &art,
            transform.translation.truncate(),
            PLAYER_HURT_RADIUS,
            Color::srgba(0.2, 0.75, 1.0, 0.45),
            18.0,
        );
    }

    for (transform, enemy) in &enemies {
        let pos = transform.translation.truncate();
        spawn_debug_circle(
            &mut commands,
            &art,
            pos,
            enemy_hurt_radius(enemy.kind, enemy.elite),
            Color::srgba(1.0, 0.2, 0.25, 0.32),
            18.1,
        );
        spawn_debug_circle(
            &mut commands,
            &art,
            pos,
            enemy_melee_reach(enemy.kind) + PLAYER_HURT_RADIUS,
            Color::srgba(1.0, 0.72, 0.15, 0.18),
            18.0,
        );
    }

    for (transform, projectile) in &projectiles {
        let radius = match projectile.owner {
            ProjectileOwner::Player => PLAYER_PROJECTILE_RADIUS,
            ProjectileOwner::Enemy => ENEMY_PROJECTILE_RADIUS,
        };
        let color = match projectile.owner {
            ProjectileOwner::Player => Color::srgba(1.0, 0.95, 0.25, 0.45),
            ProjectileOwner::Enemy => Color::srgba(1.0, 0.25, 0.35, 0.45),
        };
        spawn_debug_circle(
            &mut commands,
            &art,
            transform.translation.truncate(),
            radius,
            color,
            18.2,
        );
    }

    for slash in &slashes {
        spawn_debug_circle(
            &mut commands,
            &art,
            slash.origin,
            slash.reach,
            Color::srgba(1.0, 0.9, 0.2, 0.16),
            17.9,
        );
        spawn_debug_line(
            &mut commands,
            &art,
            slash.origin,
            slash.dir,
            slash.reach,
            Color::srgba(1.0, 0.9, 0.2, 0.45),
        );
    }

    for (transform, obstacle) in &obstacles {
        spawn_debug_circle(
            &mut commands,
            &art,
            transform.translation.truncate(),
            obstacle.radius,
            Color::srgba(0.6, 0.6, 0.7, 0.25),
            17.8,
        );
    }
}

fn spawn_debug_circle(
    commands: &mut Commands,
    art: &GameArt,
    pos: Vec2,
    radius: f32,
    color: Color,
    z: f32,
) {
    commands.spawn((
        art.image_sprite(&art.orb, Vec2::splat(radius * 2.0), color),
        Transform::from_translation(pos.extend(z)),
        CombatDebugVisual,
        RoomEntity,
    ));
}

fn spawn_debug_line(
    commands: &mut Commands,
    art: &GameArt,
    origin: Vec2,
    dir: Vec2,
    length: f32,
    color: Color,
) {
    let direction = if dir.length_squared() > 0.0 {
        dir.normalize()
    } else {
        Vec2::X
    };
    commands.spawn((
        art.image_sprite(&art.orb, Vec2::new(length, 5.0), color),
        Transform {
            translation: (origin + direction * length * 0.5).extend(18.3),
            rotation: Quat::from_rotation_z(direction.y.atan2(direction.x)),
            ..default()
        },
        CombatDebugVisual,
        RoomEntity,
    ));
}

pub fn spawn_damage_number(commands: &mut Commands, font: &Handle<Font>, pos: Vec2, amount: i32) {
    spawn_damage_number_colored(
        commands,
        font,
        pos,
        amount,
        Color::srgb(1.0, 0.95, 0.55),
    );
}

pub fn spawn_crit_damage_number(commands: &mut Commands, font: &Handle<Font>, pos: Vec2, amount: i32) {
    let drift = ((pos.x as i32 % 7) - 3) as f32 * 9.0;
    commands.spawn((
        Text2d::new(format!("CRIT! {}", amount)),
        TextFont {
            font: font.clone().into(),
            font_size: FontSize::Px(32.0),
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.8, 0.2)),
        Transform::from_translation(pos.extend(6.0)),
        DamageNumber {
            life: Timer::from_seconds(0.65, TimerMode::Once),
            velocity: Vec2::new(drift, 90.0),
        },
        RoomEntity,
    ));
}

pub fn spawn_damage_number_colored(
    commands: &mut Commands,
    font: &Handle<Font>,
    pos: Vec2,
    amount: i32,
    color: Color,
) {
    let drift = ((pos.x as i32 % 7) - 3) as f32 * 9.0;
    commands.spawn((
        Text2d::new(format!("{amount}")),
        TextFont {
            font: font.clone().into(),
            font_size: FontSize::Px(22.0),
            ..default()
        },
        TextColor(color),
        Transform::from_translation(pos.extend(6.0)),
        DamageNumber {
            life: Timer::from_seconds(0.55, TimerMode::Once),
            velocity: Vec2::new(drift, 72.0),
        },
        RoomEntity,
    ));
}

pub fn update_damage_numbers(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut TextColor, &mut DamageNumber)>,
) {
    let dt = time.delta_secs();
    for (entity, mut transform, mut color, mut number) in &mut query {
        number.life.tick(time.delta());
        if number.life.is_finished() {
            commands.entity(entity).despawn();
            continue;
        }
        transform.translation.x += number.velocity.x * dt;
        transform.translation.y += number.velocity.y * dt;
        color.0 = color.0.with_alpha(number.life.fraction_remaining());
    }
}
