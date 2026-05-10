use crate::{camera::PlayerCamera, components::physics_components::Ground, utils::Raycast};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player)
            .add_systems(Update, (move_player, handle_grapple_dbg));
    }
}

#[derive(Default, PartialEq, PartialOrd)]
pub enum PlayerState {
    #[default]
    InGame = 0,
    InMenu,
}

#[derive(Component)]
pub struct Player {
    pub move_speed: f32,
    pub motion_speed: f32,
    pub pitch: f32,
    pub state: PlayerState,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            move_speed: 1.0,
            motion_speed: 0.1,
            pitch: 0.0,
            state: PlayerState::default(),
        }
    }
}

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
            Player::default(),
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
    let Ok((entity, mut ext_impulse, transform, player)) = query.single_mut() else {
        return;
    };
    let Ok(ctx) = rapier_context.single() else {
        return;
    };

    let mut input_dir = Vec3::ZERO;
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

    if keyboard.pressed(KeyCode::Space) && is_floored(entity, transform, &ctx, &ground_query) {
        impulse_dir.y += 5.0;
    }

    if impulse_dir != Vec3::ZERO {
        ext_impulse.impulse += impulse_dir * player.move_speed;
    }
}

fn is_floored(
    player: Entity,
    transform: &Transform,
    ctx: &RapierContext,
    grounds: &Query<(), With<Ground>>,
) -> bool {
    let filter = QueryFilter::default().exclude_collider(player);
    if let Some((hit, _)) = ctx.cast_ray(transform.translation, Vec3::NEG_Y, 1.1, true, filter) {
        grounds.contains(hit)
    } else {
        false
    }
}

fn handle_grapple_dbg(
    mouse: Res<ButtonInput<MouseButton>>,
    mut gizmos: Gizmos,
    query: Query<(&Transform, &Player, Entity)>,
    rapier_context: ReadRapierContext,
) {
    if !mouse.pressed(MouseButton::Right) {
        return;
    }
    let Ok((transform, player, entity)) = query.single() else {
        return;
    };
    let Ok(ctx) = rapier_context.single() else {
        return;
    };

    let origin = transform.translation + Vec3::Y * 0.5;
    let direction = (transform.rotation * Quat::from_rotation_x(player.pitch) * Vec3::NEG_Z)
        .normalize_or_zero();

    if let Some(hit) = Raycast::new(origin, direction, &ctx, Some(50.0), Some(vec![entity])) {
        gizmos.line(origin, hit.hit_point, Color::WHITE);
    }
}
