use bevy::prelude::*;
use bevy::render::mesh::PrimitiveTopology;
use bevy::render::render_asset::RenderAssetUsages;
use bevy_rapier3d::prelude::*;
use rand::Rng;

use crate::features::ball::*;

fn create_checkerboard_mesh(
    meshes: &mut ResMut<Assets<Mesh>>,
    width: usize,
    length: usize,
    tile_size: f32,
) -> Handle<Mesh> {
    let half_width = (width as f32 * tile_size) / 2.0;
    let half_length = (length as f32 * tile_size) / 2.0;

    let mut positions = Vec::new();
    let mut indices = Vec::new();
    let mut normals = Vec::new();
    let mut colors = Vec::new();

    for x in 0..width {
        for z in 0..length {
            let x_pos = x as f32 * tile_size - half_width;
            let z_pos = z as f32 * tile_size - half_length;
            let base_index = (x * length + z) as u32 * 4;

            positions.extend_from_slice(&[
                [x_pos, 0.0, z_pos],                         // Bottom left
                [x_pos + tile_size, 0.0, z_pos],             // Bottom right
                [x_pos + tile_size, 0.0, z_pos + tile_size], // Top right
                [x_pos, 0.0, z_pos + tile_size],             // Top left
            ]);

            normals.extend_from_slice(&[
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ]);

            indices.extend_from_slice(&[
                base_index,
                base_index + 1,
                base_index + 2,
                base_index,
                base_index + 2,
                base_index + 3,
            ]);

            let color = if (x + z) % 2 == 0 {
                [1.0, 1.0, 1.0, 1.0]
            } else {
                [0.8, 0.8, 0.8, 1.0]
            };
            colors.extend_from_slice(&[color, color, color, color]);
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));

    meshes.add(mesh)
}

pub fn setup_scene(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let mut rng = rand::thread_rng();

    // Add a light source
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            color: Color::srgb(1.0, 0.9, 0.9),
            shadows_enabled: true,
            intensity: 60_000_000.,
            range: 100_000.0,
            shadow_depth_bias: 0.2,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 30.0, 0.0),
        ..default()
    });

    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.3, 0.3, 0.3), // Set a subtle gray color for overall light
        brightness: 5_000.0, // Adjust brightness to provide a softer base illumination
    });

    let tile_size = 1.0;
    let width = 25; // Number of tiles along the width
    let length = 25; // Number of tiles along the length

    let mesh_handle = create_checkerboard_mesh(&mut meshes, width, length, tile_size);

    let colors = [
        Color::srgba_u8(38, 70, 83, 100),    // Blue
        Color::srgba_u8(233, 196, 106, 100), // Yellow
        Color::srgba_u8(42, 157, 143, 100),  // Green
        Color::srgba_u8(231, 111, 81, 100),  // Red
    ];
    commands
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

        Ball::spawn(
            ball_radius,
            position,
            velocity,
            class,
            &mut commands,
            &mut meshes,
            &mut materials,
        );
    }

    ControllableBall::spawn(
        Vec3::new(0.0, 0.0, 0.0),
        &mut commands,
        &mut meshes,
        &mut materials,
    );
}

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
