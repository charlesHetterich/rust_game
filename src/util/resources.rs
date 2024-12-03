use bevy::prelude::Resource;
use bevy::prelude::Timer;

#[derive(Resource, Default, Debug)]
pub struct ProgramInputs {
    pub headless: bool,
    pub ai_control: bool,
}

#[derive(Resource)]
pub struct SimulationTimer {
    pub timer: Timer,
}
