mod game;

use bevy::prelude::*;
use game::GamePlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Catevy".into(),
                        resolution: (1180u32, 700u32).into(),
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
            GamePlugin,
        ))
        .run();
}
