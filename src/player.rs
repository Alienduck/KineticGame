use crate::{camera::PlayerCamera, map::Ground, state::PlayerState, utils::Raycast};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player)
            .add_systems(Update, (move_player, handle_grapple));
    }
}

#[derive(Component)]
pub struct Player {
    pub move_speed: f32,
    pub air_speed: f32,
    pub jump_force: f32,
    pub motion_speed: f32,
    pub pitch: f32,
    pub state: PlayerState,
    pub grapple_anchor: Option<Vec3>,
    pub grapple_force: f32,
    pub grapple_range: f32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            move_speed: 2.0,
            air_speed: 1.0,
            jump_force: 20.0,
            motion_speed: 0.1,
            pitch: 0.0,
            state: PlayerState::default(),
            grapple_anchor: None,
            grapple_force: 5.0,
            grapple_range: 200.0,
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
            (
                Mesh3d(meshes.add(Capsule3d::new(0.5, 1.0))),
                MeshMaterial3d(materials.add(Color::srgb(0.2, 0.8, 0.2))),
                Transform::from_xyz(0.0, 100.0, 0.0),
                Player::default(),
            ),
            (
                RigidBody::Dynamic,
                GravityScale(5.0),
                Collider::capsule_y(0.5, 0.5),
                CollisionGroups::new(Group::GROUP_1, Group::ALL),
                Ccd::enabled(),
                ColliderMassProperties::Density(2.0),
                LockedAxes::ROTATION_LOCKED,
            ),
            (
                Friction {
                    coefficient: 0.0,
                    combine_rule: CoefficientCombineRule::Min,
                },
                Restitution {
                    coefficient: 0.0,
                    combine_rule: CoefficientCombineRule::Min,
                },
                ExternalImpulse::default(),
                Damping {
                    linear_damping: 0.2,
                    angular_damping: 0.5,
                },
                Velocity::default(),
            ),
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
    mut query: Query<(
        Entity,
        &mut ExternalImpulse,
        &mut Damping,
        &Transform,
        &Player,
    )>,
    ground_query: Query<(), With<Ground>>,
    rapier_context: ReadRapierContext,
) {
    let Ok((entity, mut ext_impulse, mut damping, transform, player)) = query.single_mut() else {
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

    let is_floored = is_floored(entity, transform, &ctx, &ground_query);

    if is_floored {
        damping.linear_damping = 5.0;
    } else {
        damping.linear_damping = 0.5;
    }

    let mut impulse_dir = Vec3::ZERO;

    if input_dir != Vec3::ZERO {
        let forward = transform.forward();
        let right = transform.right();
        let flat_forward = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
        let flat_right = Vec3::new(right.x, 0.0, right.z).normalize_or_zero();

        let move_direction =
            (flat_forward * -input_dir.z + flat_right * input_dir.x).normalize_or_zero();

        let current_speed = if is_floored {
            player.move_speed
        } else {
            player.air_speed
        };
        impulse_dir += move_direction * current_speed;
    }

    if keyboard.pressed(KeyCode::Space) && is_floored {
        impulse_dir.y += player.jump_force;
    }

    if impulse_dir != Vec3::ZERO {
        ext_impulse.impulse += impulse_dir;
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

fn handle_grapple(
    mouse: Res<ButtonInput<MouseButton>>,
    mut query: Query<(Entity, &mut Player, &Transform, &mut ExternalImpulse)>,
    rapier_context: ReadRapierContext,
    mut gizmos: Gizmos,
) {
    let Ok((entity, mut player, transform, mut ext_impulse)) = query.single_mut() else {
        return;
    };
    let Ok(ctx) = rapier_context.single() else {
        return;
    };

    if mouse.just_released(MouseButton::Right) {
        player.grapple_anchor = None;
        return;
    }

    let origin = transform.translation + Vec3::Y * 0.5;

    if mouse.just_pressed(MouseButton::Right) {
        let direction = (transform.rotation * Quat::from_rotation_x(player.pitch) * Vec3::NEG_Z)
            .normalize_or_zero();
        if let Some(hit) = Raycast::new(origin, direction, &ctx, Some(50.0), Some(vec![entity])) {
            player.grapple_anchor = Some(hit.hit_point);
        }
    }

    if mouse.pressed(MouseButton::Right) {
        if let Some(anchor) = player.grapple_anchor {
            gizmos.line(origin, anchor, Color::WHITE);
            let pull_dir = (anchor - transform.translation).normalize_or_zero();
            ext_impulse.impulse += pull_dir * player.grapple_force;
        }
    }
}
