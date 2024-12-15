use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_rapier3d::render::DebugRenderContext;

use crate::features::system::*;
use crate::features::system_controls::toggle_debug_render;

pub fn setup_graphics(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 50.0, 40.0)
                .looking_at(Vec3::new(0.0, 0., 5.0), Vec3::Y),
            ..Default::default()
        },
        CameraController,
    ));
}

pub fn start_cursor_toggle_grab(
    mut q_windows: Query<&mut Window, With<PrimaryWindow>>,
    debug_render_state: ResMut<DebugRenderContext>,
) {
    cursor_toggle_grab(&mut q_windows.single_mut().cursor);
    toggle_debug_render(debug_render_state);
}

pub fn setup_ui(mut commands: Commands) {
    commands
        .spawn(NodeBundle {
            style: Style {
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
                    ..Default::default()
                },
                Outline {
                    width: Val::Px(1.),
                    offset: Val::Px(1.),
                    color: Color::srgb(0.5, 0.5, 0.5), // grey
                },
            ));
        });
}
