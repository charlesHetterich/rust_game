use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use nn::ModuleT;
use tch::*;

use crate::features::ball::*;
use crate::modeling::ModelResource;
use crate::scenes::ball_game_scene::BallGameScene;

pub enum ControllerType {
    Keyboard,
    AI { training: bool },
}

/// Human player input
fn get_keyboard_input(keyboard_input: &Res<ButtonInput<KeyCode>>) -> (bool, bool, bool, bool) {
    let up = keyboard_input.pressed(KeyCode::ArrowUp);
    let down = keyboard_input.pressed(KeyCode::ArrowDown);
    let left = keyboard_input.pressed(KeyCode::ArrowLeft);
    let right = keyboard_input.pressed(KeyCode::ArrowRight);
    (up, down, left, right)
}

/// Gets movement actions from model over a batch of states
fn get_ai_movement(model: &TrainableCModule, s: Tensor) -> Vec<(bool, bool, bool, bool)> {
    // Run model
    let a_next = model
        .forward_t(&s, false)
        .sigmoid()
        .to_device(Device::Cpu)
        .to_kind(Kind::Float)
        .gt(0.5);

    // extract output & return
    Vec::<Vec<bool>>::try_from(a_next)
        .unwrap()
        .iter()
        .map(|x| (x[0], x[1], x[2], x[3]))
        .collect::<Vec<(bool, bool, bool, bool)>>()
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

// collect AI input
fn push(velocity: &Velocity, transform: &Transform, target_quadrant: Option<(i8, i8)>) -> Vec<f32> {
    let mut inputs = Vec::new();
    inputs.push(velocity.linvel.x);
    inputs.push(velocity.linvel.z);
    inputs.push(transform.translation.x);
    inputs.push(transform.translation.z);
    if let Some((v1, v2)) = target_quadrant {
        inputs.append(&mut vec![v1 as f32, v2 as f32]);
    }
    inputs
}
pub fn move_balls(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    model_resource: Res<ModelResource>,
    scene_query: Query<&BallGameScene>,
    balls_query: Query<(&Velocity, &Transform, &Ball), Without<ControllableBall>>,
    mut pball_query: Query<(&mut Velocity, &Transform), With<ControllableBall>>,
    time: Res<Time>,
) {
    // collect model input filtered for AI controlled scenes
    let batch_states = Tensor::cat(
        &(scene_query
            .iter()
            .filter_map(|scene| match &scene.controller {
                ControllerType::Keyboard => None,
                ControllerType::AI { training: _ } => {
                    let (p_velocity, p_transform) = pball_query.get_mut(scene.player_ball).unwrap();
                    let mut inputs = Vec::new();
                    inputs.append(&mut push(&*p_velocity, p_transform, None));
                    for (velocity, transform, ball) in balls_query.iter_many(&scene.game_balls) {
                        inputs.append(&mut push(velocity, transform, ball.class.target_quadrant()));
                    }
                    Some(Tensor::from_slice(&inputs).view([1, (50 + 1) * 6 - 2]))
                }
            })
            .collect::<Vec<Tensor>>()),
        0,
    );

    // run model and apply movements
    let batch_actions: Vec<(bool, bool, bool, bool)> =
        get_ai_movement(&model_resource.model, batch_states);
    let mut i = 0;
    for scene in scene_query.iter() {
        let (mut p_velocity, _) = pball_query.get_mut(scene.player_ball).unwrap();
        let action = match &scene.controller {
            ControllerType::Keyboard => get_keyboard_input(&keyboard_input),
            ControllerType::AI { training: _ } => {
                i += 1;
                batch_actions[i - 1]
            }
        };
        apply_movement(
            action.0,
            action.1,
            action.2,
            action.3,
            &mut p_velocity,
            &time,
        );
    }
}
