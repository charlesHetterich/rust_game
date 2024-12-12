use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum BallTag {
    Red,
    Blue,
    Green,
    Yellow,
    Player,
}

impl BallTag {
    pub fn color(&self) -> Color {
        match self {
            BallTag::Blue => Color::srgb_u8(38, 70, 83),      // Blue
            BallTag::Yellow => Color::srgb_u8(233, 196, 106), // Yellow
            BallTag::Green => Color::srgb_u8(42, 157, 143),   // Green
            BallTag::Red => Color::srgb_u8(231, 111, 81),     // Red
            BallTag::Player => Color::WHITE,
        }
    }

    pub fn target_quadrant(&self) -> Option<(i8, i8)> {
        match self {
            BallTag::Blue => Some((-1, -1)),
            BallTag::Yellow => Some((1, -1)),
            BallTag::Green => Some((-1, 1)),
            BallTag::Red => Some((1, 1)),
            BallTag::Player => None,
        }
    }
}

#[derive(Component)]
pub struct Ball {
    pub drag_coefficient: f32,
    pub class: BallTag,
}
impl Ball {
    pub fn spawn(
        radius: f32,
        position: Vec3,
        velocity: Vec3,
        tag: BallTag,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
    ) -> Entity {
        let entity = commands
            .spawn((
                // Rendering components
                PbrBundle {
                    mesh: meshes.add(Mesh::from(Sphere { radius: radius })),
                    material: materials.add(StandardMaterial {
                        base_color: tag.color(),
                        ..Default::default()
                    }),
                    transform: Transform::from_translation(position),
                    ..Default::default()
                },
                // Physics components
                Collider::ball(radius),
                RigidBody::Dynamic,
                Restitution {
                    coefficient: if tag == BallTag::Player { 0.0 } else { 0.7 },
                    combine_rule: CoefficientCombineRule::Average,
                },
                Velocity::linear(velocity),
                LockedAxes::TRANSLATION_LOCKED_Y,
                // Other
                Ball {
                    drag_coefficient: if tag == BallTag::Player { 0.1 } else { 0.01 },
                    class: tag,
                },
            ))
            .id();

        entity
    }

    /// Determines if a ball is the the quadrant
    /// associated with its class
    pub fn correct_quadrant(&self, x: f32, z: f32) -> bool {
        if self.class == BallTag::Player {
            false
        } else {
            let (t_x, t_z) = self.class.target_quadrant().unwrap();
            x * t_x as f32 > 0. && z * t_z as f32 > 0.
        }
    }
}

#[derive(Component)]
pub struct ControllableBall {}
impl ControllableBall {
    pub fn spawn(
        position: Vec3,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
    ) {
        let ball_entity = Ball::spawn(
            1.5,
            position,
            Vec3::ZERO,
            BallTag::Player,
            commands,
            meshes,
            materials,
        );
        commands.entity(ball_entity).insert(ControllableBall {});
    }
}
