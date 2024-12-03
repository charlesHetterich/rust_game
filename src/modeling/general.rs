use bevy::prelude::*;
use tch::*;

// Define a resource to store the model
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
