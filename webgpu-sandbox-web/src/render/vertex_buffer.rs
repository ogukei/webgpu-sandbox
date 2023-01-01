
use std::sync::Arc;

use crate::render::device::Device;

use web_sys::{
    GpuBufferDescriptor,
    gpu_buffer_usage,
    GpuBuffer,
};

pub struct VertexBuffer {
    device: Arc<Device>,
    vertex_count: usize,
    vertex_buffer: GpuBuffer,
}

impl VertexBuffer {
    pub fn new(device: &Arc<Device>, points: Vec<f32>) -> Arc<Self> {
        let vertex_count = points.len() / 3;
        let mut vertex_buffer_descriptor = GpuBufferDescriptor::new(
            (std::mem::size_of::<f32>() * points.len()) as f64,
            gpu_buffer_usage::VERTEX);
        vertex_buffer_descriptor.mapped_at_creation(true);
        let vertex_buffer = device.device().create_buffer(&vertex_buffer_descriptor);
        let vertex_array = js_sys::Float32Array::new(&vertex_buffer.get_mapped_range());
        vertex_array.copy_from(&points);
        vertex_buffer.unmap();
        let this = Self {
            vertex_count,
            vertex_buffer,
            device: Arc::clone(device),
        };
        Arc::new(this)
    }

    pub fn vertex_count(&self) -> usize {
        self.vertex_count
    }

    pub fn buffer(&self) -> &GpuBuffer {
        &self.vertex_buffer
    }
}
