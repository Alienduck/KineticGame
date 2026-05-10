use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use kinetic_game::{camera::CameraPlugin, map::MapPlugin, menu::MenuPlugin, player::PlayerPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                fit_canvas_to_parent: true,
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins((MapPlugin, PlayerPlugin, CameraPlugin, MenuPlugin))
        .run();
}
