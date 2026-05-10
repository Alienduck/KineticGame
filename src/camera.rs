use crate::{
    player::{Player, PlayerState},
    state::AppState,
};
use bevy::{input::mouse::MouseMotion, prelude::*};
use std::f32::consts::FRAC_PI_2;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, move_camera.run_if(in_state(AppState::InGame)));
    }
}

#[derive(Component)]
pub struct PlayerCamera;

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
    if player.state == PlayerState::InMenu {
        return;
    }

    for msg in mouse_motion.read() {
        transform.rotate_y(-msg.delta.x * time.delta_secs() * player.motion_speed);
        player.pitch = (player.pitch - msg.delta.y * time.delta_secs() * player.motion_speed)
            .clamp(-FRAC_PI_2 + 0.01, FRAC_PI_2 - 0.01);
        cam_transform.rotation = Quat::from_rotation_x(player.pitch);
    }
}
