use bevy::prelude::*;
use std::fs::File;
use std::io::{BufWriter, Write};

use crate::features::ball::*;

#[derive(Resource, Default)]
pub struct AggBallPositions {
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

pub fn track_ball_positions(
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
