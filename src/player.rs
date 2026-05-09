use std::f32::consts::FRAC_PI_2;

use bevy::{input::mouse::MouseMotion, prelude::*};
use bevy_rapier3d::prelude::*;

use crate::{components::physics_components::Ground, utils::Raycast};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player).add_systems(
            Update,
            (
                move_camera,
                move_player,
                auto_add_ground_to_map,
                cleanup_gltf_cameras,
                handle_right_click,
            ),
        );
    }
}

#[derive(Component, Default)]
pub struct Player {
    move_speed: f32,
    motion_speed: f32,
    pitch: f32,
}

#[derive(Component)]
pub struct PlayerCamera;

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn((
            Mesh3d(meshes.add(Capsule3d::new(0.5, 1.0))),
            MeshMaterial3d(materials.add(Color::srgb(0.2, 0.8, 0.2))),
            Transform::from_xyz(0.0, 10.0, 0.0),
            RigidBody::Dynamic,
            Collider::capsule_y(0.5, 0.5),
            ColliderMassProperties::Density(2.0),
            LockedAxes::ROTATION_LOCKED,
            Friction {
                coefficient: 0.2,
                combine_rule: CoefficientCombineRule::Average,
            },
            ExternalImpulse::default(),
            Damping {
                linear_damping: 2.0,
                angular_damping: 1.0,
            },
            Player {
                motion_speed: 0.1,
                move_speed: 1.0,
                pitch: 0.0,
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Camera {
                    clear_color: ClearColorConfig::Custom(Color::srgb(0.5, 0.5, 0.8)),
                    ..default()
                },
                Camera3d::default(),
                PlayerCamera,
                Transform::from_xyz(0.0, 0.5, 0.0),
            ));
        });
}

fn move_player(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(Entity, &mut ExternalImpulse, &Transform, &Player)>,
    ground_query: Query<(), With<Ground>>,
    rapier_context: ReadRapierContext,
) {
    let mut input_dir = Vec3::ZERO;

    let Ok((entity, mut ext_impulse, transform, player)) = query.single_mut() else {
        return;
    };
    let Ok(rapier_context) = rapier_context.single() else {
        return;
    };

    if keyboard.pressed(KeyCode::KeyW) {
        input_dir.z -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        input_dir.z += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        input_dir.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        input_dir.x += 1.0;
    }

    let mut impulse_dir = Vec3::ZERO;

    if input_dir != Vec3::ZERO {
        let forward = transform.forward();
        let right = transform.right();

        let flat_forward = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
        let flat_right = Vec3::new(right.x, 0.0, right.z).normalize_or_zero();

        impulse_dir = (flat_forward * -input_dir.z + flat_right * input_dir.x).normalize_or_zero();
    }

    if keyboard.pressed(KeyCode::Space)
        && is_floored(entity, transform, &rapier_context, &ground_query)
    {
        impulse_dir.y += 5.0
    }

    if impulse_dir != Vec3::ZERO {
        ext_impulse.impulse += impulse_dir * player.move_speed;
    }
}

fn move_camera(
    mut mouse_motion: MessageReader<MouseMotion>,
    mut player_query: Query<(&mut Transform, &mut Player)>,
    mut camera_query: Query<&mut Transform, (With<PlayerCamera>, Without<Player>)>,
    time: Res<Time>,
) {
    let Ok((mut transform, mut player)) = player_query.single_mut() else {
        return;
    };
    let Ok(mut cam_transform) = camera_query.single_mut() else {
        return;
    };
    for msg in mouse_motion.read() {
        transform.rotate_y(-msg.delta.x * time.delta_secs() * player.motion_speed);

        player.pitch -= msg.delta.y * time.delta_secs() * player.motion_speed;

        player.pitch = player.pitch.clamp(-FRAC_PI_2 + 0.01, FRAC_PI_2 - 0.01);

        cam_transform.rotation = Quat::from_rotation_x(player.pitch);
    }
}

fn is_floored(
    player_entity: Entity,
    player_transform: &Transform,
    rapier_context: &RapierContext,
    ground_query: &Query<(), With<Ground>>,
) -> bool {
    let origin = player_transform.translation;
    let direction = Vec3::NEG_Y;
    let max_toi = 1.1;
    let solid = true;

    let filter = QueryFilter::default().exclude_collider(player_entity);

    if let Some((hit_entity, _toi)) =
        rapier_context.cast_ray(origin, direction, max_toi, solid, filter)
    {
        ground_query.contains(hit_entity)
    } else {
        false
    }
}

fn auto_add_ground_to_map(
    mut commands: Commands,
    new_colliders: Query<Entity, Added<Collider>>,
    player_query: Query<(), With<Player>>,
) {
    for entity in new_colliders.iter() {
        if !player_query.contains(entity) {
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

fn handle_right_click(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    gizmos: Gizmos,
    query: Query<(&Transform, &Player, Entity)>,
    rapier_context: ReadRapierContext,
) {
    if mouse_buttons.pressed(MouseButton::Right) {
        draw_grapple_dbg(gizmos, query, rapier_context);
    }
}

fn draw_grapple_dbg(
    mut gizmos: Gizmos,
    query: Query<(&Transform, &Player, Entity)>,
    rapier_context: ReadRapierContext,
) {
    let Ok((transform, player, entity)) = query.single() else {
        return;
    };
    let Ok(rapier_context) = rapier_context.single() else {
        return;
    };
    let origin = transform.translation + Vec3::Y * 0.5;
    let direction = (transform.rotation * Quat::from_rotation_x(player.pitch) * Vec3::NEG_Z)
        .normalize_or_zero();
    let Some(hit_point) = Raycast::new(
        origin,
        direction,
        &rapier_context,
        Some(50.0),
        Some(vec![entity]),
    ) else {
        return;
    };
    gizmos.line(origin, hit_point.hit_point, Color::WHITE);
}
