
use nalgebra_glm as glm;

pub struct SceneContext {
    view_quat: glm::Quat,
}

impl SceneContext {
    pub fn new() -> Self {
        Self {
            view_quat: glm::quat_look_at(&glm::vec3(0.0, 0.0, 1.0), &glm::vec3(0.0, 1.0, 0.0)),
        }
    }

    pub fn view_quat(&self) -> glm::Quat {
        self.view_quat
    }

    pub fn forward_frame(&mut self, delta_time: f32) {
        let rotation_y = glm::quat_angle_axis(delta_time * glm::pi::<f32>() * 0.32, &glm::vec3(0.0, 1.0, 0.0));
        self.view_quat *= rotation_y;
    }
}
