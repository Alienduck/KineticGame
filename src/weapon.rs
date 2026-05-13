use crate::camera::PlayerCamera;
use bevy::{platform::collections::HashSet, prelude::*};
use bevy_rapier3d::prelude::*;

pub struct WeaponsPlugin;

impl Plugin for WeaponsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (shoot_projectile, handle_explosions));
    }
}

#[derive(Component)]
pub struct Projectile {
    pub radius: f32,
    pub max_force: f32,
    pub shooter: Entity,
}

fn shoot_projectile(
    mouse: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    camera_query: Query<(&GlobalTransform, &ChildOf), With<PlayerCamera>>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok((global_transform, parent)) = camera_query.single() else {
        return;
    };
    let shooter_entity = parent.parent();

    let direction = global_transform.forward().normalize_or_zero();

    let origin = global_transform.translation() + direction * 0.2;

    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.2))),
        MeshMaterial3d(materials.add(Color::srgb(0.9, 0.4, 0.1))),
        Transform::from_translation(origin),
        RigidBody::Dynamic,
        Collider::ball(0.2),
        Sensor,
        ActiveEvents::COLLISION_EVENTS,
        Velocity {
            linvel: direction * 40.0,
            angvel: Vec3::ZERO,
        },
        GravityScale(1.0),
        Ccd::enabled(),
        Projectile {
            radius: 24.0,
            max_force: 200.0,
            shooter: shooter_entity,
        },
    ));
}

fn handle_explosions(
    mut commands: Commands,
    mut collision_messages: MessageReader<CollisionEvent>,
    projectile_query: Query<(Entity, &Projectile, &Transform)>,
    mut target_query: Query<(Entity, &Transform, &mut ExternalImpulse)>,
    rapier_context: ReadRapierContext,
) {
    let Ok(ctx) = rapier_context.single() else {
        return;
    };
    let mut despawn_queue = HashSet::new();

    for message in collision_messages.read() {
        if let CollisionEvent::Started(e1, e2, _) = message {
            let (proj_entity, other_entity) = if projectile_query.contains(*e1) {
                (*e1, *e2)
            } else if projectile_query.contains(*e2) {
                (*e2, *e1)
            } else {
                continue;
            };

            if let Ok((_, projectile, _)) = projectile_query.get(proj_entity) {
                if other_entity == projectile.shooter {
                    continue;
                }
            }

            despawn_queue.insert(proj_entity);
        }
    }

    for proj_entity in despawn_queue {
        if let Ok((_, projectile, proj_transform)) = projectile_query.get(proj_entity) {
            let explosion_center = proj_transform.translation;

            for (target_e, target_t, mut target_impulse) in target_query.iter_mut() {
                let distance = explosion_center.distance(target_t.translation);
                if distance > projectile.radius || distance < 0.1 {
                    continue;
                }

                let dir_to_player = (target_t.translation - explosion_center).normalize_or_zero();

                let dir_to_bomb = -dir_to_player;

                let filter = QueryFilter::default()
                    .exclude_collider(proj_entity)
                    .exclude_collider(target_e);

                let mut has_line_of_sight = false;

                if let Some((_, hit_toi)) =
                    ctx.cast_ray(target_t.translation, dir_to_bomb, distance, true, filter)
                {
                    if hit_toi >= distance - 0.5 {
                        has_line_of_sight = true;
                    }
                } else {
                    has_line_of_sight = true;
                }

                if has_line_of_sight {
                    let falloff = 1.0 - (distance / projectile.radius).clamp(0.0, 1.0);
                    target_impulse.impulse += dir_to_player * projectile.max_force * falloff;
                }
            }
        }
        commands.entity(proj_entity).despawn();
    }
}
