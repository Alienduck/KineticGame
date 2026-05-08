use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

pub struct PhysicsCorePlugin;

impl Plugin for PhysicsCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
            .add_systems(Update, apply_impulses);
    }
}

fn apply_impulses() {}
