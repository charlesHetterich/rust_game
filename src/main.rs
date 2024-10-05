use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::render::mesh::PrimitiveTopology;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use bevy_rapier3d::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, setup_graphics)
        .add_systems(Startup, setup_physics)
        .add_systems(Update, print_ball_altitude)
        .add_systems(Update, move_camera) // Add the camera movement system
        .add_systems(Update, cursor_toggle_grab)
        .add_systems(Update, toggle_debug_render)
        .run();
}

fn toggle_debug_render(
    // mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut debug_render_state: ResMut<DebugRenderContext>,
) {
    if keyboard_input.just_pressed(KeyCode::F1) {
        debug_render_state.enabled = !debug_render_state.enabled;
    }
}

fn setup_graphics(mut commands: Commands) {
    // Add a camera so we can see the debug-render.
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-3.0, 3.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        },
        CameraController,
    ));
}

fn setup_physics(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    /* Create the bouncing ball. */
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(Sphere { radius: 0.5 })),
            material: materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 1.0, 1.0),
                ..Default::default()
            }),
            transform: Transform::from_xyz(0.0, 4.0, 0.0),
            ..Default::default()
        },
        RigidBody::Dynamic,
        Collider::ball(0.5),
        Restitution::coefficient(0.99),
    ));

    // Add a light source
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            intensity: 10_000_000.,
            range: 100.0,
            shadow_depth_bias: 0.2,
            ..default()
        },
        transform: Transform::from_xyz(8.0, 16.0, 8.0),
        ..default()
    });

    let tile_size = 1.0;
    let width = 25; // Number of tiles along the width
    let length = 25; // Number of tiles along the length

    let half_width = (width as f32 * tile_size) / 2.0;
    let half_length = (length as f32 * tile_size) / 2.0;

    // Create the ground mesh
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    let mut normals = Vec::new();
    let mut colors = Vec::new();

    for x in 0..width {
        for z in 0..length {
            let x_pos = x as f32 * tile_size - half_width;
            let z_pos = z as f32 * tile_size - half_length;

            // Each tile is a quad with two triangles
            let base_index = (x * length + z) as u32 * 4;

            positions.push([x_pos, 0.0, z_pos]); // Bottom left
            positions.push([x_pos + tile_size, 0.0, z_pos]); // Bottom right
            positions.push([x_pos + tile_size, 0.0, z_pos + tile_size]); // Top right
            positions.push([x_pos, 0.0, z_pos + tile_size]); // Top left

            // Normals for a flat plane
            normals.push([0.0, 1.0, 0.0]);
            normals.push([0.0, 1.0, 0.0]);
            normals.push([0.0, 1.0, 0.0]);
            normals.push([0.0, 1.0, 0.0]);

            indices.push(base_index);
            indices.push(base_index + 1);
            indices.push(base_index + 2);

            indices.push(base_index);
            indices.push(base_index + 2);
            indices.push(base_index + 3);

            // Checkerboard pattern
            let color = if (x + z) % 2 == 0 {
                [1.0, 1.0, 1.0, 1.0]
            } else {
                [0.8, 0.8, 0.8, 1.0]
            };
            colors.push(color);
            colors.push(color);
            colors.push(color);
            colors.push(color);
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);

    // Spawn the floor entity with the mesh
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(mesh),
            material: materials.add(StandardMaterial {
                base_color: Color::srgb(0.5, 0.8, 0.5), // Use vertex colors for the checkerboard effect
                cull_mode: None,
                ..Default::default()
            }),
            transform: Transform::from_xyz(0.0, -20.0, 0.0),
            ..Default::default()
        },
        Collider::cuboid(half_width, 0.1, half_length),
    ));

    /* Create the ground. */
    // commands.spawn((
    //     PbrBundle {
    //         mesh: meshes.add(Mesh::from(Cuboid {
    //             half_size: Vec3::new(25.0, 0.1, 25.0),
    //         })),
    //         material: materials.add(StandardMaterial {
    //             base_color: Color::srgb(0.3, 0.5, 0.3),
    //             ..Default::default()
    //         }),
    //         transform: Transform::from_xyz(0.0, -20.0, 0.0),
    //         ..Default::default()
    //     },
    //     Collider::cuboid(25.0, 0.1, 25.0),
    // ));
}

fn print_ball_altitude(mut positions: Query<&mut Transform, With<RigidBody>>) {
    for mut transform in positions.iter_mut() {
        dbg!(transform.rotation.to_axis_angle());
        transform.rotation = Quat::from_rotation_z(270_f32.to_radians());
        //println!("Ball altitude: {}", transform.translation.y);
    }
}

// Camera movement
#[derive(Component)]
struct CameraController;

fn move_camera(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut query: Query<&mut Transform, With<CameraController>>,
) {
    for mut transform in query.iter_mut() {
        // Handle keyboard input for movement
        let forward = Vec3::new(transform.forward().x, 0.0, transform.forward().z).normalize();
        let right = Vec3::new(transform.right().x, 0.0, transform.right().z).normalize();
        let up = Vec3::Y;

        if keyboard_input.pressed(KeyCode::KeyW) {
            transform.translation += forward * 0.1; // Move the camera forward
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            transform.translation -= forward * 0.1; // Move the camera backward
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            transform.translation -= right * 0.1; // Move the camera left
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            transform.translation += right * 0.1; // Move the camera right
        }
        if keyboard_input.pressed(KeyCode::ShiftLeft) {
            transform.translation -= up * 0.1; // Move the camera down
        }
        if keyboard_input.pressed(KeyCode::Space) {
            transform.translation += up * 0.1; // Move the camera up
        }

        // Handle mouse motion for rotation
        for event in mouse_motion_events.read() {
            let delta_x = event.delta.x * 0.1; // Adjust sensitivity as needed
            let delta_y = event.delta.y * 0.1; // Adjust sensitivity as needed

            // Rotate around the Y axis (yaw)
            transform.rotation = Quat::from_rotation_y(-delta_x.to_radians()) * transform.rotation;

            // Rotate around the X axis (pitch)
            transform.rotation = transform.rotation * Quat::from_rotation_x(-delta_y.to_radians());
        }
    }
}
fn cursor_toggle_grab(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut q_windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        let mut primary_window = q_windows.single_mut();
        match primary_window.cursor.grab_mode {
            CursorGrabMode::None => {
                primary_window.cursor.grab_mode = CursorGrabMode::Locked;
                primary_window.cursor.visible = false;
            }
            _ => {
                primary_window.cursor.grab_mode = CursorGrabMode::None;
                primary_window.cursor.visible = true;
            }
        }
    }
}
