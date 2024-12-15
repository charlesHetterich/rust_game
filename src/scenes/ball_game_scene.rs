use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::Rng;

use crate::features::ball::*;
use crate::features::player_controllers::ControllerType;
use crate::modeling::Trajectory;

use super::general;

/// manages data collection of each game
#[derive(Component)]
pub struct BallGameScene {
    pub trajectory: Trajectory,
    pub game_balls: Vec<Entity>,
    pub player_ball: Entity,
    pub controller: ControllerType,
}

/// Creates a set of ball game scenes
pub fn setup_world(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // create stages
    let grid_size = 6;
    for i in 0..grid_size {
        for j in 0..grid_size {
            let x = (i as f32 - grid_size as f32 / 2.0) * 60.0;
            let z = (j as f32 - grid_size as f32 / 2.0) * 60.0;
            setup_scene(
                &mut commands,
                Vec3::new(x, 0.0, z),
                &mut materials,
                &mut meshes,
            );
        }
    }

    // add light
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.3, 0.3, 0.3),
        brightness: 25_000.0,
    });
}

fn setup_scene(
    commands: &mut Commands,
    center: Vec3, // Add this parameter
    materials: &mut ResMut<Assets<StandardMaterial>>,
    meshes: &mut ResMut<Assets<Mesh>>,
) {
    let parent_entity = commands
        .spawn((
            Transform::from_translation(center),
            GlobalTransform::IDENTITY,
        ))
        .id();

    let mut scene = None;
    commands.entity(parent_entity).with_children(|parent| {
        scene = Some(_setup_scene(parent, materials, meshes));
    });
    commands.entity(parent_entity).insert(scene.unwrap());
}

fn _setup_scene(
    parent: &mut ChildBuilder,
    mut materials: &mut ResMut<Assets<StandardMaterial>>,
    mut meshes: &mut ResMut<Assets<Mesh>>,
) -> BallGameScene {
    let tile_size = 1.0;
    let width = 25; // Number of tiles along the width
    let length = 25; // Number of tiles along the length

    let mesh_handle = general::create_checkerboard_mesh(meshes, width, length, tile_size);

    let colors = [
        Color::srgba_u8(38, 70, 83, 100),    // Blue
        Color::srgba_u8(233, 196, 106, 100), // Yellow
        Color::srgba_u8(42, 157, 143, 100),  // Green
        Color::srgba_u8(231, 111, 81, 100),  // Red
    ];
    parent
        .spawn((TransformBundle::default(), InheritedVisibility::VISIBLE)) // Parent entity
        .with_children(|parent| {
            let total_width: f32 = width as f32 * tile_size;
            let total_length = length as f32 * tile_size;
            for (i, &base_color) in colors.iter().enumerate() {
                let x_offset = (i % 2) as f32 * total_width - (total_width / 2.0); // Adjust x position
                let z_offset = (i / 2) as f32 * total_length - (total_length / 2.0); // Adjust z position

                parent.spawn(PbrBundle {
                    mesh: mesh_handle.clone(), // Reuse the handle, not the mesh data
                    material: materials.add(StandardMaterial {
                        base_color,
                        alpha_mode: AlphaMode::Blend,
                        cull_mode: None,
                        ..Default::default()
                    }),
                    transform: Transform::from_xyz(x_offset, 0.0, z_offset),
                    ..Default::default()
                });
            }

            // Spawning walls
            let wall_material = materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 1.0, 1.0), // White walls
                alpha_mode: AlphaMode::Opaque,
                ..Default::default()
            });

            let wall_height = 1.0; // Example height of the walls
            let wall_thickness = 0.3; // Thickness of the walls

            // Define positions and sizes for each wall
            let walls = [
                (
                    Transform::from_xyz(0.0, 0.0, -total_length - wall_thickness),
                    total_width,
                    wall_height,
                    wall_thickness,
                ), // North wall
                (
                    Transform::from_xyz(0.0, 0.0, total_length + wall_thickness),
                    total_width,
                    wall_height,
                    wall_thickness,
                ), // South wall
                (
                    Transform::from_xyz(-total_width - wall_thickness, 0.0, 0.0),
                    wall_thickness,
                    wall_height,
                    total_length,
                ), // West wall
                (
                    Transform::from_xyz(total_width + wall_thickness, 0.0, 0.0),
                    wall_thickness,
                    wall_height,
                    total_length,
                ), // East wall
            ];

            for (transform, width, height, depth) in walls.iter() {
                parent
                    .spawn(PbrBundle {
                        mesh: meshes.add(Mesh::from(Cuboid {
                            half_size: Vec3::new(*width, *height, *depth),
                        })),
                        material: wall_material.clone(),
                        transform: *transform,
                        ..Default::default()
                    })
                    .insert((
                        Collider::cuboid(*width, *height, *depth),
                        Restitution::coefficient(1.0),
                    ));
            }
        });

    // spawn random balls
    let mut rng = rand::thread_rng();
    let mut game_balls = Vec::new();
    let num_balls = 50;
    let ball_radius = 0.5;
    for _ in 0..num_balls {
        let x_max = width as f32 * tile_size / 2.0 - ball_radius * 2.0;
        let z_max = length as f32 * tile_size / 2.0 - ball_radius * 2.0;
        let x_pos = rng.gen_range(-x_max..x_max);
        let z_pos = rng.gen_range(-z_max..z_max);

        let position = Vec3::new(x_pos, 0.0, z_pos);
        let velocity = Vec3::new(0.0, 0.0, 0.0);
        let class = match rng.gen_range(0..4) {
            0 => BallTag::Red,
            1 => BallTag::Blue,
            2 => BallTag::Green,
            _ => BallTag::Yellow,
        };

        game_balls.push(
            Ball::spawn(
                ball_radius,
                position,
                velocity,
                class,
                parent,
                &mut meshes,
                &mut materials,
            )
            .id(),
        );
    }

    let player_ball = ControllableBall::spawn(
        Vec3::new(0.0, 0.0, 0.0),
        // commands,
        parent,
        &mut meshes,
        &mut materials,
    );

    BallGameScene {
        trajectory: Trajectory::new(),
        game_balls,
        player_ball,
        controller: ControllerType::AI { training: true },
        // controller: ControllerType::Keyboard,
    }
    // how do add scene as a component to parent entity??
}

/// Resets balls back to random starting position
pub fn reset_scene(
    mut param_set: ParamSet<(
        Query<(&mut Velocity, &mut Transform), With<Ball>>,
        Query<(&mut Velocity, &mut Transform), With<ControllableBall>>,
    )>,
) {
    let mut rng = rand::thread_rng();
    let width = 25.0; // Assuming the width of the area
    let length = 25.0; // Assuming the length of the area

    // reset other balls
    for (mut velocity, mut transform) in param_set.p0().iter_mut() {
        let x_max = width / 2.0 - 1.0;
        let z_max = length / 2.0 - 1.0;
        let x_pos = rng.gen_range(-x_max..x_max);
        let z_pos = rng.gen_range(-z_max..z_max);

        transform.translation = Vec3::new(x_pos, 0.0, z_pos);
        velocity.linvel = Vec3::new(0.0, 0.0, 0.0);
    }

    // Reset controllable ball
    for (mut velocity, mut transform) in param_set.p1().iter_mut() {
        transform.translation = Vec3::new(0.0, 0.0, 0.0);
        velocity.linvel = Vec3::new(0.0, 0.0, 0.0);
    }
}
