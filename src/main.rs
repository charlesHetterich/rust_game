use std::env;
use std::time::Duration;

use balltrainer::util::events::SimulationEndedEvent;
use balltrainer::util::playdata::on_simulation_end;
use balltrainer::util::resources::update_world_state;
use balltrainer::util::resources::WorldState;
use bevy::app::ScheduleRunnerPlugin;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use balltrainer::features::ball::*;
use balltrainer::features::player_controllers::*;
use balltrainer::features::system::*;
use balltrainer::features::ui::*;
use balltrainer::modeling::load_model;
use balltrainer::scenes::BallGameScene;

use balltrainer::util::logging::*;
use balltrainer::util::monitoring::print_fps_system;
use balltrainer::util::playdata::check_simulation_end;
use balltrainer::util::resources::{ProgramInputs, SimulationTimer};
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
        .add_plugins(FrameTimeDiagnosticsPlugin::default()) // monitor fps
        //add events
        .add_event::<SimulationEndedEvent>()
        // add resources
        // .insert_resource(Sett)
        .insert_resource(WorldState::new())
        .insert_resource(SimulationTimer {
            timer: Timer::from_seconds(15.0, TimerMode::Repeating),
        })
        // startup systems
        .add_systems(Startup, setup_graphics)
        .add_systems(Startup, BallGameScene::setup_world)
        .add_systems(Startup, setup_ui)
        .add_systems(Startup, start_cursor_toggle_grab)
        .add_systems(Startup, load_model)
        // update systems
        .add_systems(Update, print_fps_system)
        .add_systems(Update, apply_ball_drag)
        .add_systems(Update, check_simulation_end)
        .add_systems(Update, on_simulation_end)
        .add_systems(Update, update_world_state)
        .add_systems(Update, move_balls);

    // headless setup
    if (&program_inputs).headless {
        app.insert_resource(AggBallPositions::default())
            // .add_systems(Update, move_player_w_ai)
            .add_systems(Update, track_ball_positions)
            .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
                1.0 / 60.0,
            )));
    // regular setup
    } else {
        app.insert_resource(AggBallPositions::default())
            .add_systems(Update, apply_system_inputs);
    }

    // rest of general setup
    app.insert_resource(program_inputs);
    app.run();
}
