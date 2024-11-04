use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::{Cursor, CursorGrabMode, PrimaryWindow};
use bevy_rapier3d::prelude::*;

use super::objects::*;
use super::scenes::ball_game_scene::reset_scene;

#[derive(Component)]
pub struct CameraController;

pub fn apply_system_inputs(
    // mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_motion_events: EventReader<MouseMotion>,
    mut q_windows: Query<&mut Window, With<PrimaryWindow>>,
    mut param_set: ParamSet<(
        Query<&mut Transform, With<CameraController>>,
        ParamSet<(
            Query<(&mut Velocity, &mut Transform), With<Ball>>,
            Query<(&mut Velocity, &mut Transform), With<ControllableBall>>,
        )>, // To get the state of all balls, excluding ControllableBall
    )>,
    debug_render_state: ResMut<DebugRenderContext>,
) {
    let mut primary_window = q_windows.single_mut();

    // toggle rapier debug render
    if (&keyboard_input).just_pressed(KeyCode::F2) {
        toggle_debug_render(debug_render_state)
    }

    // toggle camera control & mouse lock
    if (&keyboard_input).just_pressed(KeyCode::Escape) {
        cursor_toggle_grab(&mut primary_window.cursor);
    }

    // move camera
    if primary_window.cursor.grab_mode == CursorGrabMode::Locked {
        move_camera(
            &keyboard_input,
            mouse_motion_events,
            (&mut param_set.p0()).single_mut(),
        );
    }

    // reset scene
    if (&keyboard_input).just_pressed(KeyCode::KeyR) {
        reset_scene(param_set.p1());
    }
}

fn toggle_debug_render(
    // mut commands: Commands,
    mut debug_render_state: ResMut<DebugRenderContext>,
) {
    debug_render_state.enabled = !debug_render_state.enabled;
}

fn move_camera(
    keyboard_input: &Res<ButtonInput<KeyCode>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut camera_transform: Mut<'_, Transform>,
) {
    let transform = &mut camera_transform;

    let forward = Vec3::new(transform.forward().x, 0.0, transform.forward().z).normalize();
    let right = Vec3::new(transform.right().x, 0.0, transform.right().z).normalize();
    let up = Vec3::Y;

    if keyboard_input.pressed(KeyCode::KeyW) {
        transform.translation += forward * 0.1; // Move the camera forward
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        transform.translation -= forward * 0.1; // Move the camera backward
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        transform.translation -= right * 0.1; // Move the camera left
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        transform.translation += right * 0.1; // Move the camera right
    }
    if keyboard_input.pressed(KeyCode::ShiftLeft) {
        transform.translation -= up * 0.1; // Move the camera down
    }
    if keyboard_input.pressed(KeyCode::Space) {
        transform.translation += up * 0.1; // Move the camera up
    }

    // Handle mouse motion for rotation
    for event in mouse_motion_events.read() {
        let delta_x = event.delta.x * 0.1; // Adjust sensitivity as needed
        let delta_y = event.delta.y * 0.1; // Adjust sensitivity as needed

        // Rotate around the Y axis (yaw)
        transform.rotation = Quat::from_rotation_y(-delta_x.to_radians()) * transform.rotation;

        // Rotate around the X axis (pitch)
        transform.rotation = transform.rotation * Quat::from_rotation_x(-delta_y.to_radians());
    }
}

pub fn cursor_toggle_grab(cursor: &mut Cursor) {
    match cursor.grab_mode {
        CursorGrabMode::None => {
            cursor.grab_mode = CursorGrabMode::Locked;
            cursor.visible = false;
        }
        _ => {
            cursor.grab_mode = CursorGrabMode::None;
            cursor.visible = true;
        }
    }
}
