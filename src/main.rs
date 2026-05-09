use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use kinetic_game::player::PlayerPlugin;

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
        .add_plugins(PlayerPlugin)
        .add_systems(Startup, setup_scene)
        .run();
}

fn setup_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(0.0, 50.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    let map_handle: Handle<Scene> = asset_server.load("3Dmodels/fps_tps_map.glb#Scene0");

    commands.spawn((
        SceneRoot(map_handle),
        Transform::default(),
        AsyncSceneCollider {
            shape: Some(ComputedColliderShape::TriMesh(TriMeshFlags::default())),
            ..default()
        },
    ));
}
