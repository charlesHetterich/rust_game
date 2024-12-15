use bevy::prelude::*;
use tch::*;

#[derive(Component)]
pub struct Trajectory {
    // pub steps: Vec<(Tensor, f32, f32)>,
    pub state: Vec<Tensor>,
    pub action: Vec<f32>,
    pub reward: Vec<f32>,
}

// * i think we do `unsafe` thing bc
// * tensors are C and not rust compliant ?
unsafe impl Sync for Trajectory {}
impl Trajectory {
    pub fn new() -> Self {
        Trajectory {
            state: Vec::new(),
            action: Vec::new(),
            reward: Vec::new(),
        }
    }
}

#[derive(Resource)]
pub struct ModelResource {
    pub model: TrainableCModule,
    pub _vs: nn::VarStore,
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
pub fn load_model(mut commands: Commands) {
    let model_resource = ModelResource::new("src/modeling/ball_policy.pt");
    commands.insert_resource(model_resource);
}
