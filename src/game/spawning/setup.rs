use crate::game::assets::{GameArt, Sfx};
use crate::game::state::{RoomKind, RunState};
use crate::game::progression::room_kind_for;
use crate::game::components::*;
use bevy::audio::{PlaybackSettings, Volume};
use bevy::prelude::*;

use super::actors::spawn_player;

// Z-layer constants
pub const FLOOR_Z: f32 = -20.0;
pub const WALL_Z: f32 = -10.0;
pub const SHADOW_Z: f32 = 1.0;
pub const ACTOR_Z: f32 = 2.0;

pub fn setup_world(
    mut commands: Commands,
    art: Res<GameArt>,
    sfx: Res<Sfx>,
    mut run: ResMut<RunState>,
) {
    commands.spawn(Camera2d);
    commands.spawn((
        AudioPlayer::new(sfx.music.clone()),
        PlaybackSettings::LOOP.with_volume(Volume::Linear(0.28)),
    ));
    spawn_walls(&mut commands, &art);
    spawn_vignette(&mut commands, &art);
    spawn_player(&mut commands, &art);
    run.current_room = room_kind_for(&run);
    super::rooms::spawn_room(&mut commands, &art, &mut run);
    spawn_hud(&mut commands, &art);
}

fn spawn_walls(commands: &mut Commands, art: &GameArt) {
    commands.spawn((
        art.image_sprite(&art.wall, Vec2::new(1180.0, 700.0), Color::WHITE),
        Transform::from_translation(Vec3::new(0.0, 0.0, WALL_Z)),
    ));
}

fn spawn_vignette(commands: &mut Commands, art: &GameArt) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        ImageNode::new(art.vignette.clone()),
    ));
}

pub fn spawn_floor(commands: &mut Commands, art: &GameArt, kind: RoomKind) {
    let tint = match kind {
        RoomKind::Combat => Color::srgb(0.62, 0.60, 0.70),
        RoomKind::Rest => Color::srgb(0.52, 0.70, 0.58),
        RoomKind::Treasure => Color::srgb(0.74, 0.66, 0.50),
        RoomKind::Boss => Color::srgb(0.78, 0.46, 0.46),
    };
    commands.spawn((
        art.image_sprite(&art.floor, Vec2::new(1120.0, 640.0), tint),
        Transform::from_translation(Vec3::new(0.0, 0.0, FLOOR_Z)),
        RoomEntity,
    ));
}

fn spawn_hud(commands: &mut Commands, art: &GameArt) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(18.0),
                top: Val::Px(16.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.05, 0.05, 0.08).with_alpha(0.45)),
        ))
        .with_child((
            Text::new("placeholder"),
            TextFont {
                font: art.font.clone().into(),
                font_size: FontSize::Px(20.0),
                ..default()
            },
            TextColor(Color::srgb(0.95, 0.96, 1.0)),
            HudText,
        ));
}