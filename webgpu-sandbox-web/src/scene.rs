
use nalgebra_glm as glm;

use std::sync::{Arc, Mutex};

use crate::{
    Model,
};

pub struct SceneContext {
    state: Mutex<SceneState>,
    model: Arc<Model>,
}

impl SceneContext {
    pub fn new(model: &Arc<Model>) -> Arc<SceneContext> {
        let this = Self {
            state: Mutex::new(SceneState::new()),
            model: Arc::clone(model),
        };
        Arc::new(this)
    }

    pub fn forward_frame(&self, delta_time: f32) {
        let Ok(mut state) = self.state.lock() else { return };
        state.forward_frame(delta_time);
    }

    pub fn view_quat(&self) -> glm::Quat {
        let Ok(state) = self.state.lock() else { return glm::quat_identity() };
        state.view_quat()
    }

    pub fn model(&self) -> &Arc<Model> {
        &self.model
    }
}

struct SceneState {
    view_quat: glm::Quat,
}

impl SceneState {
    pub fn new() -> Self {
        let this = Self {
            view_quat: glm::quat_identity(),
        };
        this
    }

    pub fn view_quat(&self) -> glm::Quat {
        self.view_quat
    }

    pub fn forward_frame(&mut self, delta_time: f32) {
        let rotation_y = glm::quat_angle_axis(delta_time * glm::pi::<f32>() * 0.32, &glm::vec3(0.0, 1.0, 0.0));
        self.view_quat *= rotation_y;
    }
}
