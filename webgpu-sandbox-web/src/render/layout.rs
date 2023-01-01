
use std::sync::Arc;

use crate::render::device::Device;

use web_sys::{
    GpuPipelineLayout,
};

pub struct PipelineLayout {
    device: Arc<Device>,
    layout: GpuPipelineLayout,
}

impl PipelineLayout {
    pub fn new(device: &Arc<Device>, layout: GpuPipelineLayout) -> Arc<Self> {
        let this = Self {
            device: Arc::clone(device),
            layout,
        };
        Arc::new(this)
    }

    pub fn layout(&self) -> &GpuPipelineLayout {
        &self.layout
    }
}
