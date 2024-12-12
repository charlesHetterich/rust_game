use bevy::prelude::*;
use bevy_rapier3d::prelude::Velocity;

use balltrainer::{features::ball::Ball, util::playdata};

fn main() {
    let ball_query = Query::<(With<Velocity>, With<Transform>, With<Ball>)>::default();

    // add fake data (50 balls)
    for _ in 0..50 {
        ball_query
    }

    let output = playdata::collect_ai_input(ball_query);

    // let val = true;
    // print!("The value is: {}", val as i32 as f32);
}
