use crate::{camera::PlayerCamera, components::physics_components::Ground, player::Player};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_scene)
            .add_systems(Update, (auto_add_ground_to_map, cleanup_gltf_cameras));
    }
}

fn setup_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(0.0, 50.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        SceneRoot(asset_server.load("3Dmodels/fps_tps_map.glb#Scene0")),
        Transform::default(),
        AsyncSceneCollider {
            shape: Some(ComputedColliderShape::TriMesh(TriMeshFlags::default())),
            ..default()
        },
    ));
}

fn auto_add_ground_to_map(
    mut commands: Commands,
    new_colliders: Query<Entity, Added<Collider>>,
    players: Query<(), With<Player>>,
) {
    for entity in new_colliders.iter() {
        if !players.contains(entity) {
            commands.entity(entity).insert(Ground);
        }
    }
}

fn cleanup_gltf_cameras(
    mut commands: Commands,
    intruder_cameras: Query<Entity, (With<Camera>, Without<PlayerCamera>)>,
) {
    for entity in intruder_cameras.iter() {
        commands.entity(entity).despawn();
    }
}
