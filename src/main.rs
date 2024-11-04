use std::time::Duration;

use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_rapier3d::prelude::*;
use nn::ModuleT;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use tch::*;

mod system_controls;
use system_controls::*;

mod objects;
use objects::*;

mod scenes;

#[derive(Resource, Default, Debug)]
struct ProgramInputs {
    headless: bool,
    ai_control: bool,
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        // startup systems
        .add_systems(Startup, general_setup)
        .add_systems(Startup, setup_graphics)
        .add_systems(Startup, scenes::BallGameScene::setup_scene)
        .add_systems(Startup, setup_ui) // Add the UI setup system
        .add_systems(Startup, start_cursor_toggle_grab)
        .add_systems(Startup, load_model)
        // update systems
        .add_systems(Update, apply_ball_drag);

    // capture program inputs
    let args: Vec<String> = env::args().collect();
    let program_inputs = ProgramInputs {
        headless: args.contains(&"--headless".to_string()),
        ai_control: args.contains(&"--ai-control".to_string()),
    };

    // headless setup
    if (&program_inputs).headless {
        app.insert_resource(AggBallPositions::default())
            .add_systems(Update, check_simulation_end)
            .add_systems(Update, move_controllable_ball_with_ai)
            .add_systems(Update, track_ball_positions)
            .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
                1.0 / 60.0,
            )));
    // regular setup
    } else {
        app.add_systems(Update, apply_system_inputs);
        if (&program_inputs).ai_control {
            app.add_systems(Update, move_controllable_ball_with_ai);
        } else {
            app.add_systems(Update, move_controllable_ball_with_keyboard);
        }
    }

    // rest of general setup
    app.insert_resource(program_inputs);
    app.run();
}

#[derive(Resource)]
struct SimulationTimer {
    timer: Timer,
}

#[derive(Resource, Default)]
struct AggBallPositions {
    positions: std::collections::HashMap<Entity, Vec<(f32, f32)>>,
}
impl AggBallPositions {
    pub fn save_to_file(
        &self,
        file_path: &str,
        ball_classes: &Query<&Ball>,
    ) -> std::io::Result<()> {
        let file = File::create(file_path)?;
        let mut writer = BufWriter::new(file);

        for (entity, positions) in &self.positions {
            if let Ok(ball) = ball_classes.get(*entity) {
                writeln!(writer, "Class: {:?}", ball.class)?;
                for (x, z) in positions {
                    writeln!(writer, "{},{}", x, z)?;
                }
                writeln!(writer, "---")?; // Separator between balls
            }
        }

        writer.flush()?;
        Ok(())
    }
}

fn track_ball_positions(
    mut ball_positions: Option<ResMut<AggBallPositions>>,
    query: Query<(Entity, &Transform), With<Ball>>,
) {
    if let Some(ball_positions) = &mut ball_positions {
        for (entity, transform) in query.iter() {
            let position = (transform.translation.x, transform.translation.z);
            ball_positions
                .positions
                .entry(entity)
                .or_insert_with(Vec::new)
                .push(position);
        }
    }
}

// Define a resource to store the model
#[derive(Resource)]
pub struct ModelResource {
    model: TrainableCModule,
    _vs: nn::VarStore,
}

impl ModelResource {
    pub fn new(model_path: &str) -> Self {
        let vs = nn::VarStore::new(Device::Cpu);
        let mut model =
            TrainableCModule::load(model_path, vs.root()).expect("Failed to load model");
        model.set_eval();
        ModelResource { model, _vs: vs }
    }
}
fn load_model(mut commands: Commands) {
    let model_resource = ModelResource::new("src/modeling/ball_policy.pt");
    commands.insert_resource(model_resource);
}

fn check_simulation_end(
    mut writer: EventWriter<AppExit>,
    time: Res<Time>,
    mut simulation_timer: ResMut<SimulationTimer>,
    ball_positions: Res<AggBallPositions>,
    ball_query: Query<&Ball>,
) {
    // Update the timer with the elapsed time
    if simulation_timer.timer.tick(time.delta()).finished() {
        // Save the ball positions to a file
        let save_result = ball_positions.save_to_file("ball_positions.txt", &ball_query);

        match save_result {
            Ok(_) => println!("Ball positions saved successfully."),
            Err(e) => eprintln!("Failed to save ball positions: {}", e),
        }

        // Exit the app by sending an exit event
        writer.send(AppExit::Success);
    }
}

fn general_setup(mut commands: Commands) {
    // TODO : add a bunch of the other more basic setup stuff here
    // Add 10s game timer
    commands.insert_resource(SimulationTimer {
        timer: Timer::from_seconds(1.0, TimerMode::Once),
    });
}

fn setup_graphics(mut commands: Commands) {
    // Add a camera so we can see the debug-render.
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 50.0, 40.0)
                .looking_at(Vec3::new(0.0, 0., 5.0), Vec3::Y),
            ..Default::default()
        },
        CameraController,
    ));
}

fn apply_ball_drag(mut query: Query<(&mut Velocity, &Ball), With<Ball>>) {
    for (mut velocity, ball) in query.iter_mut() {
        // let drag_coefficient = 0.1; // Adjust as needed
        let drag_force = -velocity.linvel * ball.drag_coefficient;
        velocity.linvel += drag_force;
    }
}

fn apply_movement(
    direction_up: bool,
    direction_down: bool,
    direction_left: bool,
    direction_right: bool,
    velocity: &mut Velocity,
    time: &Res<Time>,
) {
    let mut direction = Vec3::ZERO;

    if direction_up {
        direction.z -= 1.0;
    }
    if direction_down {
        direction.z += 1.0;
    }
    if direction_left {
        direction.x -= 1.0;
    }
    if direction_right {
        direction.x += 1.0;
    }

    if direction.length_squared() > 0.0 {
        direction = direction.normalize() * 250.0;
    }

    velocity.linvel += direction * time.delta_seconds();
}

// Function to get movement based on keyboard input
fn get_keyboard_input(keyboard_input: &Res<ButtonInput<KeyCode>>) -> (bool, bool, bool, bool) {
    let up = keyboard_input.pressed(KeyCode::ArrowUp);
    let down = keyboard_input.pressed(KeyCode::ArrowDown);
    let left = keyboard_input.pressed(KeyCode::ArrowLeft);
    let right = keyboard_input.pressed(KeyCode::ArrowRight);

    (up, down, left, right)
}

// Keyboard-based player movement
fn move_controllable_ball_with_keyboard(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Velocity, &mut Transform), With<ControllableBall>>,
    time: Res<Time>, // To make movement frame rate independent
) {
    if let Ok((mut velocity, _)) = query.get_single_mut() {
        let (up, down, left, right) = get_keyboard_input(&keyboard_input);
        apply_movement(up, down, left, right, &mut velocity, &time);
    }
}

// Function to get movement based on AI output
fn get_ai_movement(model: &TrainableCModule, input: Tensor) -> (bool, bool, bool, bool) {
    let output = model.forward_t(&input, false);
    let output = output.sigmoid(); // Apply sigmoid function

    // Extract the 4 float values (logits) from the tensor
    let output_values = Vec::<f32>::try_from(
        output
            .view([-1])
            .to_device(Device::Cpu)
            .to_kind(Kind::Float),
    )
    .unwrap();

    // The AI model's output will be an array of 4 values
    let up = output_values[0] > 0.5;
    let down = output_values[1] > 0.5;
    let left = output_values[2] > 0.5;
    let right = output_values[3] > 0.5;

    (up, down, left, right)
}

// AI-based player movement
fn move_controllable_ball_with_ai(
    model_resource: Res<ModelResource>,
    mut param_set: ParamSet<(
        Query<&mut Velocity, With<ControllableBall>>,
        Query<(&Velocity, &Transform), With<Ball>>, // To get the state of all balls, excluding ControllableBall
    )>,
    time: Res<Time>, // To make movement frame rate independent
) {
    // Step 1: Gather the AI input by borrowing from the second query (non-mutably)
    let mut inputs = Vec::new();
    {
        let ball_query = param_set.p1(); // Immutable borrow for reading ball states
        for (ball_velocity, ball_transform) in ball_query.iter() {
            inputs.push(ball_velocity.linvel.x);
            inputs.push(ball_velocity.linvel.z);
            inputs.push(ball_transform.translation.x);
            inputs.push(ball_transform.translation.z);
        }
    }

    // Step 2: Pass the input to the AI model
    let input_tensor = Tensor::from_slice(&inputs).view([1, (50 + 1) * 4]); // Adjust for batch size of 1
    let (up, down, left, right) = get_ai_movement(&model_resource.model, input_tensor);

    // Step 3: Apply movement by borrowing the first query mutably
    if let Ok(mut velocity) = param_set.p0().get_single_mut() {
        // println!("{:?}", transform.translation.x);
        apply_movement(up, down, left, right, &mut velocity, &time);
    }
}

fn start_cursor_toggle_grab(mut q_windows: Query<&mut Window, With<PrimaryWindow>>) {
    cursor_toggle_grab(&mut q_windows.single_mut().cursor);
}

fn setup_ui(mut commands: Commands) {
    commands
        .spawn(NodeBundle {
            style: Style {
                // size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                margin: UiRect::all(Val::Px(25.0)),
                align_self: AlignSelf::Stretch,
                justify_self: JustifySelf::Stretch,
                flex_wrap: FlexWrap::Wrap,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                align_content: AlignContent::Center,
                ..Default::default()
            },
            background_color: Color::NONE.into(),
            // material: materials.add(Color::NONE.into()),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Px(10.0),
                        height: Val::Px(10.0),
                        ..Default::default()
                    },
                    border_radius: BorderRadius::all(Val::Px(5.0)),
                    // background_color: Color::WHITE.into(),
                    ..Default::default()
                },
                Outline {
                    width: Val::Px(1.),
                    offset: Val::Px(1.),
                    // grey color
                    color: Color::srgb(0.5, 0.5, 0.5),
                },
            ));
        });
}

// ##########################################################################################
