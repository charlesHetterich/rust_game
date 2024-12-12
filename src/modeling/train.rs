use super::{ModelResource, Trajectory};
use tch::Tensor;

/// trains model on batch of trajectories using REINFORCE algorithm
/// ** in prog **
pub fn learn(res: &mut ModelResource, trajectories: Vec<&Trajectory>) {
    for trajectory in trajectories.iter() {
        // learn from the trajectory
        // learn(&mut model, trajectory);
        // states =
        let states = Tensor::stack(&trajectory.state, 0);
        let actions = Tensor::from_slice(&trajectory.action).unsqueeze(1);
        let rewrds = Tensor::from_slice(&trajectory.reward).unsqueeze(1);
        // let states = trajectory.state.iter().map(|s| s).collect::<Vec<_>>();
        println!("Learning! num steps: {}", trajectory.state.len());
        // for ((s, a), r) in trajectory
        //     .state
        //     .iter()
        //     .zip(trajectory.action.iter())
        //     .zip(trajectory.reward.iter())
        // {
        //     // Process each (state, action, reward) tuple
        //     // Example: learn_from_tuple(state, action, reward);
        // }
    }
}
