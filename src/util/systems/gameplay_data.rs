use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::features::ball::{Ball, ControllableBall};
use crate::scenes::BallGameScene::reset_scene;
use crate::util::logging::AggBallPositions;
use crate::util::resources::SimulationTimer;

pub fn check_simulation_end(
    time: Res<Time>,
    mut simulation_timer: ResMut<SimulationTimer>,
    ball_positions: Res<AggBallPositions>,
    ball_query: Query<&Ball>,
    param_set: ParamSet<(
        Query<(&mut Velocity, &mut Transform), With<Ball>>,
        Query<(&mut Velocity, &mut Transform), With<ControllableBall>>,
    )>,
) {
    // Update the timer with the elapsed time
    if simulation_timer.timer.tick(time.delta()).finished() {
        // Save the ball positions to a file
        let save_result = ball_positions.save_to_file("ball_positions.txt", &ball_query);

        match save_result {
            Ok(_) => println!("Ball positions saved successfully."),
            Err(e) => eprintln!("Failed to save ball positions: {}", e),
        }

        // Reset scene (or exit)
        reset_scene(param_set);
        // writer.send(AppExit::Success);
    }
}
