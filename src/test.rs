use bevy::{
    prelude::*,
    render::{render_asset::RenderAssetUsages, render_resource::*, view::RenderLayers},
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    window::WindowPlugin,
};
// use clap::Parser;

// #[derive(Parser, Debug)]
// #[command(author, version, about, long_about = None)]
// struct Args {
//     /// Run in headless mode
//     #[arg(long)]
//     headless: bool,
// }

#[derive(Component)]
struct Triangle;

#[derive(Component)]
struct RenderTexture;

fn main() {
    let headless_mode = std::env::args().any(|arg| arg == "--headless");

    let mut app: App = App::new();

    if headless_mode {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: None,
            ..default()
        }));
    } else {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                name: Some("bevy.app".into()),
                resolution: (500., 500.).into(),
                ..default()
            }),
            ..default()
        }));
    }

    app.add_systems(Startup, setup)
        .add_systems(Update, (spin_triangle, save_render_texture))
        .run();
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
) {
    // Create the render texture
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size: Extent3d {
                width: 500,
                height: 500,
                ..default()
            },
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };
    image.texture_descriptor.usage = TextureUsages::COPY_DST
        | TextureUsages::STORAGE_BINDING
        | TextureUsages::TEXTURE_BINDING
        | TextureUsages::RENDER_ATTACHMENT;
    let image_handle = images.add(image);

    // Spawn camera with render layers
    let layer = RenderLayers::layer(1);
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                target: bevy::render::camera::RenderTarget::Image(image_handle.clone()),
                ..default()
            },
            ..default()
        },
        layer.clone(),
    ));

    // Spawn the triangle with the same render layer
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Triangle2d::new(
                Vec2::Y * 50.0,
                Vec2::new(-50.0, -50.0),
                Vec2::new(50.0, -50.0),
            ))),
            material: materials.add(Color::srgb(0.8, 0.7, 0.6)),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        Triangle,
        layer,
    ));

    // Store the image handle for saving later
    commands.spawn((RenderTexture, image_handle));
}

fn spin_triangle(time: Res<Time>, mut query: Query<(&Triangle, &mut Transform)>) {
    for (_, mut transform) in query.iter_mut() {
        transform.rotation = Quat::from_rotation_z(time.delta_seconds() * 2.0) * transform.rotation;
    }
}

fn save_render_texture(
    images: Res<Assets<Image>>,
    query: Query<(&RenderTexture, &Handle<Image>)>,
    time: Res<Time>,
) {
    // Save every second
    if time.elapsed_seconds() as i32 % 1 == 0 {
        for (_, image_handle) in query.iter() {
            if let Some(image) = images.get(image_handle) {
                image
                    .clone()
                    .try_into_dynamic()
                    .expect("Failed to convert image")
                    .save(format!("frame_{}.png", time.elapsed_seconds() as i32))
                    .expect("Failed to save image");
            }
        }
    }
}
