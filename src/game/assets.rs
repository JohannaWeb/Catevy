use super::components::EnemyKind;
use bevy::audio::AudioSource;
use bevy::prelude::*;

/// Sound-effect handles, loaded once and played on demand.
#[derive(Resource, Clone)]
pub struct Sfx {
    pub swing: Handle<AudioSource>,
    pub hit: Handle<AudioSource>,
    pub enemy_death: Handle<AudioSource>,
    pub pickup: Handle<AudioSource>,
    pub dash: Handle<AudioSource>,
    pub explosion: Handle<AudioSource>,
    pub hurt: Handle<AudioSource>,
    pub music: Handle<AudioSource>,
}

pub const CAT_FRAME_SIZE: UVec2 = UVec2::new(40, 40);
pub const CAT_COLS: u32 = 4;
pub const CAT_ROWS: u32 = 2;

// Animation clips inside the (enemy) cat sheet (frame index ranges, inclusive).
pub const CAT_IDLE: (usize, usize) = (0, 3);
pub const CAT_WALK: (usize, usize) = (4, 7);

// Meow-Knight player sheet: 10 cols x 3 rows, 34x25 cells.
pub const KNIGHT_FRAME_SIZE: UVec2 = UVec2::new(34, 25);
pub const KNIGHT_COLS: u32 = 10;
pub const KNIGHT_ROWS: u32 = 3;
pub const KNIGHT_IDLE: (usize, usize) = (0, 5);
pub const KNIGHT_RUN: (usize, usize) = (10, 17);
pub const KNIGHT_ATTACK: (usize, usize) = (20, 29);

/// A repacked monster sheet: 8 cols x 3 rows (idle / walk / attack).
#[derive(Clone)]
pub struct MonsterArt {
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
    pub cell: Vec2,
    pub idle: (usize, usize),
    pub walk: (usize, usize),
    pub attack: (usize, usize),
}

impl MonsterArt {
    pub fn sprite(&self, index: usize, scale: f32, color: Color) -> Sprite {
        Sprite {
            image: self.image.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: self.layout.clone(),
                index,
            }),
            color,
            custom_size: Some(self.cell * scale),
            ..default()
        }
    }
}

#[derive(Resource, Clone)]
pub struct GameArt {
    pub cat: Handle<Image>,
    pub cat_layout: Handle<TextureAtlasLayout>,
    pub knight: Handle<Image>,
    pub knight_layout: Handle<TextureAtlasLayout>,
    pub goblin: MonsterArt,
    pub skeleton: MonsterArt,
    pub flying_eye: MonsterArt,
    pub mushroom: MonsterArt,
    pub shadow: Handle<Image>,
    pub orb: Handle<Image>,
    pub heart: Handle<Image>,
    pub gem: Handle<Image>,
    pub floor: Handle<Image>,
    pub wall: Handle<Image>,
    pub vignette: Handle<Image>,
    pub font: Handle<Font>,
    pub slash: Handle<Image>,
    pub swords: [Handle<Image>; 6],
}

impl GameArt {
    /// A cat sprite from the shared actor sheet, tinted and scaled.
    pub fn cat_sprite(&self, index: usize, size: f32, color: Color) -> Sprite {
        Sprite {
            image: self.cat.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: self.cat_layout.clone(),
                index,
            }),
            color,
            custom_size: Some(Vec2::splat(size)),
            ..default()
        }
    }

    /// The monster sheet to use for an enemy kind, if it is a monster (not a cat).
    pub fn monster_for(&self, kind: EnemyKind) -> Option<&MonsterArt> {
        Some(match kind {
            // Goblin family (small fast enemies)
            EnemyKind::Bomber => &self.goblin,
            EnemyKind::Kitten => &self.goblin,

            // Skeleton family (charging/aggressive enemies)
            EnemyKind::Charger => &self.skeleton,
            EnemyKind::Hunter => &self.skeleton,
            EnemyKind::Scratcher => &self.skeleton,

            // Flying Eye family (ranged/magic enemies)
            EnemyKind::Caster => &self.flying_eye,
            EnemyKind::Seeker => &self.flying_eye,
            EnemyKind::ShadowCat => &self.flying_eye,
            EnemyKind::FlyingCat => &self.flying_eye,

            // Mushroom family (tanky/special enemies)
            EnemyKind::Summoner => &self.mushroom,
            EnemyKind::Bruiser => &self.mushroom,
            EnemyKind::Chonker => &self.mushroom,
            EnemyKind::Goliath => &self.mushroom,
            EnemyKind::NecromancerCat => &self.mushroom,
            EnemyKind::ShieldBearer => &self.mushroom,

            // Splitter and MimicChest use cat sprites
            EnemyKind::Splitter => return None,
            EnemyKind::MimicChest => return None,

            // Boss has its own sprite
            EnemyKind::Boss => return None,
        })
    }

    /// Get the tint color for an enemy type
    pub fn enemy_color(kind: EnemyKind) -> Color {
        match kind {
            // Goblin family - greens and yellows
            EnemyKind::Bomber => Color::srgb(1.0, 0.4, 0.3),
            EnemyKind::Kitten => Color::srgb(0.8, 0.6, 0.4),

            // Skeleton family - whites and grays
            EnemyKind::Charger => Color::srgb(0.9, 0.9, 1.0),
            EnemyKind::Hunter => Color::srgb(0.7, 0.7, 0.8),
            EnemyKind::Scratcher => Color::srgb(0.85, 0.75, 0.7),

            // Flying Eye family - purples and blues
            EnemyKind::Caster => Color::srgb(0.7, 0.5, 1.0),
            EnemyKind::Seeker => Color::srgb(0.5, 0.6, 1.0),
            EnemyKind::ShadowCat => Color::srgb(0.3, 0.2, 0.4),
            EnemyKind::FlyingCat => Color::srgb(0.6, 0.8, 1.0),

            // Mushroom family - earth tones
            EnemyKind::Summoner => Color::srgb(0.7, 0.5, 0.3),
            EnemyKind::Bruiser => Color::srgb(0.8, 0.6, 0.4),
            EnemyKind::Chonker => Color::srgb(0.9, 0.7, 0.5),
            EnemyKind::Goliath => Color::srgb(0.6, 0.4, 0.3),
            EnemyKind::NecromancerCat => Color::srgb(0.5, 0.3, 0.6),
            EnemyKind::ShieldBearer => Color::srgb(0.5, 0.5, 0.6),

            // Splitter family - cats
            EnemyKind::Splitter => Color::srgb(0.6, 0.7, 0.6),

            // Mimic
            EnemyKind::MimicChest => Color::srgb(0.9, 0.7, 0.3),

            // Boss
            EnemyKind::Boss => Color::srgb(1.0, 0.4, 0.35),
        }
    }

    /// A Meow-Knight sprite from the player sheet. `scale` multiplies the 34x25 cell.
    pub fn knight_sprite(&self, index: usize, scale: f32, color: Color) -> Sprite {
        Sprite {
            image: self.knight.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: self.knight_layout.clone(),
                index,
            }),
            color,
            custom_size: Some(Vec2::new(
                KNIGHT_FRAME_SIZE.x as f32 * scale,
                KNIGHT_FRAME_SIZE.y as f32 * scale,
            )),
            ..default()
        }
    }

    /// A plain image sprite (orb, pickup, floor, ...), tinted and sized.
    pub fn image_sprite(&self, image: &Handle<Image>, size: Vec2, color: Color) -> Sprite {
        Sprite {
            image: image.clone(),
            color,
            custom_size: Some(size),
            ..default()
        }
    }
}

pub fn setup_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let cat = asset_server.load("generated/cat-actor.png");
    let cat_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        CAT_FRAME_SIZE,
        CAT_COLS,
        CAT_ROWS,
        None,
        None,
    ));

    let knight = asset_server.load("generated/meow-knight.png");
    let knight_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        KNIGHT_FRAME_SIZE,
        KNIGHT_COLS,
        KNIGHT_ROWS,
        None,
        None,
    ));

    let mut monster = |file: &'static str,
                       cell: (u32, u32),
                       idle_n: usize,
                       walk_n: usize,
                       attack_n: usize|
     -> MonsterArt {
        MonsterArt {
            image: asset_server.load(file),
            layout: texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
                UVec2::new(cell.0, cell.1),
                8,
                3,
                None,
                None,
            )),
            cell: Vec2::new(cell.0 as f32, cell.1 as f32),
            idle: (0, idle_n - 1),
            walk: (8, 8 + walk_n - 1),
            attack: (16, 16 + attack_n - 1),
        }
    };
    let goblin = monster("generated/monster_goblin.png", (95, 53), 4, 8, 8);
    let skeleton = monster("generated/monster_skeleton.png", (96, 64), 4, 4, 8);
    let flying_eye = monster("generated/monster_flying_eye.png", (49, 39), 8, 8, 8);
    let mushroom = monster("generated/monster_mushroom.png", (64, 52), 4, 8, 8);

    commands.insert_resource(GameArt {
        cat,
        cat_layout,
        knight,
        knight_layout,
        goblin,
        skeleton,
        flying_eye,
        mushroom,
        shadow: asset_server.load("generated/shadow.png"),
        orb: asset_server.load("generated/orb.png"),
        heart: asset_server.load("generated/heart.png"),
        gem: asset_server.load("generated/gem.png"),
        floor: asset_server.load("generated/floor.png"),
        wall: asset_server.load("generated/wall.png"),
        vignette: asset_server.load("generated/vignette.png"),
        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
        slash: asset_server.load("generated/slash.png"),
        swords: [
            asset_server.load("generated/sword-rusty.png"),
            asset_server.load("generated/sword-tuna.png"),
            asset_server.load("generated/sword-great.png"),
            asset_server.load("generated/sword-flame.png"),
            asset_server.load("generated/sword-nine.png"),
            asset_server.load("generated/sword-wind.png"),
        ],
    });

    commands.insert_resource(Sfx {
        swing: asset_server.load("audio/swing.wav"),
        hit: asset_server.load("audio/hit.wav"),
        enemy_death: asset_server.load("audio/enemy_death.wav"),
        pickup: asset_server.load("audio/pickup.wav"),
        dash: asset_server.load("audio/dash.wav"),
        explosion: asset_server.load("audio/explosion.wav"),
        hurt: asset_server.load("audio/hurt.wav"),
        music: asset_server.load("audio/music.wav"),
    });
}
