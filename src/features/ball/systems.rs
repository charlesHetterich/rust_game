use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use super::component::*;

pub fn apply_ball_drag(mut query: Query<(&mut Velocity, &Ball), With<Ball>>) {
    for (mut velocity, ball) in query.iter_mut() {
        let drag_force = -velocity.linvel * ball.drag_coefficient;
        velocity.linvel += drag_force;
    }
}
