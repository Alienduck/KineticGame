use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::components::physics_components::Ground;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player)
            .add_systems(Update, move_player);
    }
}

#[derive(Component)]
pub struct Player;

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.2, 0.8, 0.2))),
        Transform::from_xyz(0.0, 5.0, 0.0),
        RigidBody::Dynamic,
        Collider::cuboid(0.5, 0.5, 0.5),
        Friction {
            coefficient: 0.8,
            combine_rule: CoefficientCombineRule::Max,
        },
        ExternalImpulse::default(),
        Damping {
            linear_damping: 2.0,
            angular_damping: 1.0,
        },
        Player,
    ));
}

fn move_player(
    mut raycast: MeshRayCast, // Ajout de mut ici
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut ExternalImpulse, &Transform), With<Player>>,
    ground_query: Query<(), With<Ground>>,
) {
    let mut impulse_dir = Vec3::ZERO;

    let Ok((mut ext_impulse, player_transform)) = query.single_mut() else {
        return;
    };

    if keyboard.pressed(KeyCode::KeyW) {
        impulse_dir.z -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        impulse_dir.z += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        impulse_dir.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        impulse_dir.x += 1.0;
    }

    if keyboard.pressed(KeyCode::Space) && is_floored(player_transform, &mut raycast, &ground_query)
    {
        impulse_dir.y += 5.0
    }

    if impulse_dir != Vec3::ZERO {
        let force = 0.8;
        ext_impulse.impulse += impulse_dir.normalize() * force;
    }
}

fn is_floored(
    player_transform: &Transform,
    raycast: &mut MeshRayCast,
    ground_query: &Query<(), With<Ground>>,
) -> bool {
    let ray = Ray3d::new(player_transform.translation, Dir3::NEG_Y);
    let impacts = raycast.cast_ray(ray, &MeshRayCastSettings::default());

    let max_distance = 1.1;

    if let Some((entity, hit)) = impacts.first() {
        hit.distance <= max_distance && ground_query.contains(*entity)
    } else {
        false
    }
}
