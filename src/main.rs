use std::env;
use std::time::Duration;

use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

// local modules
mod features;
mod modeling;
mod scenes;
mod util;

use crate::features::ball::*;
use crate::features::player_controllers::*;
use crate::features::system::*;
use crate::features::ui::*;

use crate::util::logging::*;
use crate::util::playdata::check_simulation_end;
use crate::util::resources::{ProgramInputs, SimulationTimer};

use crate::modeling::load_model;

fn main() {
    // capture program inputs
    let args: Vec<String> = env::args().collect();
    let program_inputs = ProgramInputs {
        headless: args.contains(&"--headless".to_string()),
        ai_control: args.contains(&"--ai-control".to_string()),
    };

    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        // add resources
        .insert_resource(SimulationTimer {
            timer: Timer::from_seconds(5.0, TimerMode::Repeating),
        })
        // startup systems
        .add_systems(Startup, setup_graphics)
        .add_systems(Startup, scenes::BallGameScene::setup_scene)
        .add_systems(Startup, setup_ui)
        .add_systems(Startup, start_cursor_toggle_grab)
        .add_systems(Startup, load_model)
        // update systems
        .add_systems(Update, apply_ball_drag)
        .add_systems(Update, check_simulation_end);

    // headless setup
    if (&program_inputs).headless {
        app.insert_resource(AggBallPositions::default())
            .add_systems(Update, move_player_w_ai)
            .add_systems(Update, track_ball_positions)
            .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
                1.0 / 60.0,
            )));
    // regular setup
    } else {
        app.insert_resource(AggBallPositions::default())
            .add_systems(Update, apply_system_inputs);
        if (&program_inputs).ai_control {
            app.add_systems(Update, move_player_w_ai);
        } else {
            app.add_systems(Update, move_player_w_human);
        }
    }

    // rest of general setup
    app.insert_resource(program_inputs);
    app.run();
}
