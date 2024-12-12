use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use tch::Tensor;

use crate::features::ball::{Ball, ControllableBall};
use crate::modeling::{learn, ModelResource, Trajectory};
use crate::scenes::BallGameScene::reset_scene;
use crate::util::logging::AggBallPositions;
use crate::util::{
    events::SimulationEndedEvent,
    resources::{SimulationTimer, WorldState},
};

/// Collects model input
pub fn collect_ai_input(ball_query: Vec<(&Velocity, &Transform, &Ball)>) -> Tensor {
    let mut inputs = Vec::new();
    for (ball_velocity, ball_transform, ball) in ball_query {
        inputs.push(ball_velocity.linvel.x);
        inputs.push(ball_velocity.linvel.z);
        inputs.push(ball_transform.translation.x);
        inputs.push(ball_transform.translation.z);
        if let Some((v1, v2)) = ball.class.target_quadrant() {
            inputs.append(&mut vec![v1 as f32, v2 as f32]);
        }
    }
    Tensor::from_slice(&inputs).view([1, (50 + 1) * 6 - 2])
}

pub fn check_simulation_end(
    mut event_reader: EventWriter<SimulationEndedEvent>,
    time: Res<Time>,
    mut simulation_timer: ResMut<SimulationTimer>,
) {
    // Update the timer with the elapsed time
    if simulation_timer.timer.tick(time.delta()).finished() {
        event_reader.send(SimulationEndedEvent);
    }
}

/// When a simulation ends we train the model and restart
pub fn on_simulation_end(
    mut event_reader: EventReader<SimulationEndedEvent>,
    mut model: ResMut<ModelResource>,
    trajectories: Query<&Trajectory, With<Trajectory>>,
    ball_positions: Res<AggBallPositions>,
    mut world_state: ResMut<WorldState>,
    ball_query: Query<&Ball>,
    param_set: ParamSet<(
        Query<(&mut Velocity, &mut Transform), With<Ball>>,
        Query<(&mut Velocity, &mut Transform), With<ControllableBall>>,
    )>,
) {
    if event_reader.read().into_iter().count() == 0 {
        return;
    }
    // save the ball positions to a file
    let save_result = ball_positions.save_to_file("ball_positions.txt", &ball_query);
    match save_result {
        Ok(_) => println!("Ball positions saved successfully."),
        Err(e) => eprintln!("Failed to save ball positions: {}", e),
    }

    // score stuff
    println!("Final Score: {:?}", world_state);
    world_state.reset();

    // Reset scene (or exit)
    reset_scene(param_set);
    // writer.send(AppExit::Success);

    learn(&mut model, trajectories.iter().collect());
}
