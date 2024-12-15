use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::Res;

pub fn print_fps_system(diagnostics: Res<DiagnosticsStore>) {
    // TODO : update so that we display FPS on screen rather than print to console (during 'debug mode' which will toggle with F2)
    if let Some(fps) = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.smoothed())
    {
        println!("Average FPS: {:.2}", fps);
        // if let Some(average_fps) = fps.average() {}
    }
}
