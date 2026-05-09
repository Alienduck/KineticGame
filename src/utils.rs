use bevy::prelude::*;
use bevy_rapier3d::{pipeline::QueryFilter, plugin::RapierContext};

pub struct Raycast {
    pub hit: Entity,
    pub distance: f32,
    pub hit_point: Vec3,
}

impl Raycast {
    pub fn new(
        origin: Vec3,
        direction: Vec3,
        rapier_context: &RapierContext,
        max_toi: Option<f32>,
        excludes: Option<Vec<Entity>>,
    ) -> Option<Raycast> {
        let max_toi = max_toi.unwrap_or(50.0);
        let mut filter = QueryFilter::default();
        if let Some(entities) = excludes {
            for entity in entities {
                filter = filter.exclude_collider(entity);
            }
        }
        rapier_context
            .cast_ray(origin, direction, max_toi, true, filter)
            .map(|(hit, distance)| Raycast {
                hit,
                distance,
                hit_point: origin + (direction * distance),
            })
    }

    pub fn to_destination(
        origin: Vec3,
        destination: Vec3,
        rapier_context: &RapierContext,
        max_toi: Option<f32>,
        excludes: Option<Vec<Entity>>,
    ) -> Option<Raycast> {
        let direction = (destination - origin).normalize_or_zero();
        let max_toi = max_toi.unwrap_or_else(|| origin.distance(destination));
        let mut filter = QueryFilter::default();
        if let Some(entities) = excludes {
            for entity in entities {
                filter = filter.exclude_collider(entity);
            }
        }
        rapier_context
            .cast_ray(origin, direction, max_toi, true, filter)
            .map(|(hit, distance)| Raycast {
                hit,
                distance,
                hit_point: origin + (direction * distance),
            })
    }
}
