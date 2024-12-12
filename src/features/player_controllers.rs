use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use nn::ModuleT;
use tch::*;

use crate::features::ball::*;
use crate::modeling::ModelResource;
use crate::util::playdata::collect_ai_input;

/// Collects model input
// fn collect_ai_input(ball_query: Query<(&Velocity, &Transform), With<Ball>>) -> Tensor {
//     let mut inputs = Vec::new();
//     for (ball_velocity, ball_transform) in ball_query.iter() {
//         inputs.push(ball_velocity.linvel.x);
//         inputs.push(ball_velocity.linvel.z);
//         inputs.push(ball_transform.translation.x);
//         inputs.push(ball_transform.translation.z);
//     }
//     Tensor::from_slice(&inputs).view([1, (50 + 1) * 4])
// }

/// Human player input
fn get_keyboard_input(keyboard_input: &Res<ButtonInput<KeyCode>>) -> (bool, bool, bool, bool) {
    let up = keyboard_input.pressed(KeyCode::ArrowUp);
    let down = keyboard_input.pressed(KeyCode::ArrowDown);
    let left = keyboard_input.pressed(KeyCode::ArrowLeft);
    let right = keyboard_input.pressed(KeyCode::ArrowRight);
    (up, down, left, right)
}

/// AI player input
fn get_ai_movement(model: &TrainableCModule, input: Tensor) -> (bool, bool, bool, bool) {
    // Run model
    let output = model.forward_t(&input, false);
    let output = output.sigmoid();

    // Extract outputs
    let output_values = Vec::<f32>::try_from(
        output
            .view([-1])
            .to_device(Device::Cpu)
            .to_kind(Kind::Float),
    )
    .unwrap();
    let up = output_values[0] > 0.5;
    let down = output_values[1] > 0.5;
    let left = output_values[2] > 0.5;
    let right = output_values[3] > 0.5;

    // Return
    (up, down, left, right)
}

const PLAYER_SPEED: f32 = 250.0;
/// Apply movement to player ball based on input
fn apply_movement(
    direction_up: bool,
    direction_down: bool,
    direction_left: bool,
    direction_right: bool,
    velocity: &mut Velocity,
    time: &Res<Time>,
) {
    // Get normalized input direction
    let mut direction = Vec3::ZERO;
    direction.z += (direction_down as i32 - direction_up as i32) as f32;
    direction.x += (direction_right as i32 - direction_left as i32) as f32;
    if direction.length_squared() > 0.0 {
        direction = direction.normalize() * PLAYER_SPEED;
    }

    // Apply movement to player velocity
    velocity.linvel += direction * time.delta_seconds();
}

/// Keyboard-based player movement
pub fn move_player_w_human(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Velocity, &mut Transform), With<ControllableBall>>,
) {
    if let Ok((mut velocity, _)) = query.get_single_mut() {
        let (up, down, left, right) = get_keyboard_input(&keyboard_input);
        apply_movement(up, down, left, right, &mut velocity, &time);
    }
}

/// AI-based player movement
pub fn move_player_w_ai(
    time: Res<Time>,
    model_resource: Res<ModelResource>,
    mut ball_pset: ParamSet<(
        Query<&mut Velocity, With<ControllableBall>>,
        Query<(&Velocity, &Transform, &Ball), With<Ball>>,
    )>,
) {
    // Collect input
    let (up, down, left, right) = get_ai_movement(
        &model_resource.model,
        collect_ai_input(ball_pset.p1().iter().collect()),
    );
    if let Ok(mut velocity) = ball_pset.p0().get_single_mut() {
        apply_movement(up, down, left, right, &mut velocity, &time);
    }
}
