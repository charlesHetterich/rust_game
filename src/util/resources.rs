use bevy::prelude::{Query, ResMut, Resource, Timer, Transform, With};

use crate::features::ball::Ball;

#[derive(Resource, Default, Debug)]
pub struct ProgramInputs {
    pub headless: bool,
    pub ai_control: bool,
}

#[derive(Resource)]
pub struct SimulationTimer {
    pub timer: Timer,
}

////////// ** going to move from this to using `Trajectory`s and `BallGameInstance`s //////////
#[derive(Resource, Default, Debug)]
pub struct WorldState {
    // pub state: Tensor,
    pub agg_score: f32,
    pub reward: f32,
}

// impl Display for WorldState {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {}
// }
impl WorldState {
    pub fn new() -> Self {
        WorldState {
            agg_score: 0.,
            reward: 0.,
        }
    }

    pub fn reset(&mut self) {
        self.agg_score = 0.;
        self.reward = 0.;
    }
}

pub fn update_world_state(
    mut state: ResMut<WorldState>,
    ball_query: Query<(&Transform, &Ball), With<Ball>>,
) {
    let mut reward = 0.;
    for (transform, ball) in ball_query.iter() {
        if ball.correct_quadrant(transform.translation.x, transform.translation.z) {
            reward += 1.;
        }
    }

    // state.state = new_state;
    state.agg_score += reward;
    state.reward = reward;
}

////////////////////////////////////////////////////////////////////////////////
