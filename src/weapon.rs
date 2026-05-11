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
}

fn shoot_projectile(
    mouse: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    camera_query: Query<&GlobalTransform, With<PlayerCamera>>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(global_transform) = camera_query.single() else {
        return;
    };

    let direction = global_transform.forward().normalize_or_zero();
    let origin = global_transform.translation() + direction * 1.5;

    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.2))),
        MeshMaterial3d(materials.add(Color::srgb(0.9, 0.4, 0.1))),
        Transform::from_translation(origin),
        RigidBody::Dynamic,
        Collider::ball(0.2),
        CollisionGroups::new(Group::GROUP_2, Group::GROUP_1),
        ActiveEvents::COLLISION_EVENTS,
        Velocity {
            linvel: direction * 40.0,
            angvel: Vec3::ZERO,
        },
        GravityScale(1.0),
        Ccd::enabled(),
        Projectile {
            radius: 12.0,
            max_force: 200.0,
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
            let (proj_entity, _other_entity) = if projectile_query.contains(*e1) {
                (*e1, *e2)
            } else if projectile_query.contains(*e2) {
                (*e2, *e1)
            } else {
                continue;
            };

            if despawn_queue.contains(&proj_entity) {
                continue;
            }

            let Ok((_, projectile, proj_transform)) = projectile_query.get(proj_entity) else {
                continue;
            };
            let explosion_center = proj_transform.translation;

            for (target_e, target_t, mut target_impulse) in target_query.iter_mut() {
                let distance = explosion_center.distance(target_t.translation);

                if distance > projectile.radius || distance < 0.1 {
                    continue;
                }

                let dir = (target_t.translation - explosion_center).normalize_or_zero();
                let filter = QueryFilter::default().exclude_collider(proj_entity);

                if let Some((hit_e, _)) =
                    ctx.cast_ray(explosion_center, dir, distance, true, filter)
                {
                    if hit_e == target_e {
                        let falloff = 1.0 - (distance / projectile.radius);
                        target_impulse.impulse += dir * projectile.max_force * falloff;
                    }
                }
            }
            despawn_queue.insert(proj_entity);
        }
    }

    for entity in despawn_queue {
        commands.entity(entity).try_despawn();
    }
}
