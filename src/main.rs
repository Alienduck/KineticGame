use bevy::prelude::*;
use kinetic_game::physics::PhysicsCorePlugin;

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
        .add_plugins((PhysicsCorePlugin))
        .add_systems(Startup, setup_test)
        .run();
}

fn setup_test(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(0.0, 500.0, 0.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(20.0, 0.1, 20.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.1, 0.1))),
        Transform::default(),
    ));
    commands.spawn((
        Camera::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
