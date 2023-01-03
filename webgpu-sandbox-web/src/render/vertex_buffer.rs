
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


pub struct MeshBuffer {
    device: Arc<Device>,
    index_count: usize,
    position_buffer: GpuBuffer,
    index_buffer: GpuBuffer,
    normal_buffer: GpuBuffer,
}

impl MeshBuffer {
    pub fn new(device: &Arc<Device>, positions: &Vec<f32>, indices: &Vec<u32>, normals: &Vec<f32>) -> Arc<Self> {
        // positions
        let mut position_buffer_descriptor = GpuBufferDescriptor::new(
            (std::mem::size_of::<f32>() * positions.len()) as f64,
            gpu_buffer_usage::VERTEX);
        position_buffer_descriptor.mapped_at_creation(true);
        let position_buffer = device.device().create_buffer(&position_buffer_descriptor);
        let position_array = js_sys::Float32Array::new(&position_buffer.get_mapped_range());
        position_array.copy_from(&positions);
        position_buffer.unmap();
        // index
        let mut index_buffer_descriptor = GpuBufferDescriptor::new(
            (std::mem::size_of::<u32>() * indices.len()) as f64,
            gpu_buffer_usage::INDEX);
        index_buffer_descriptor.mapped_at_creation(true);
        let index_buffer = device.device().create_buffer(&index_buffer_descriptor);
        let index_array = js_sys::Uint32Array::new(&index_buffer.get_mapped_range());
        index_array.copy_from(&indices);
        index_buffer.unmap();
        // normal
        let mut normal_buffer_descriptor = GpuBufferDescriptor::new(
            (std::mem::size_of::<f32>() * normals.len()) as f64,
            gpu_buffer_usage::VERTEX);
        normal_buffer_descriptor.mapped_at_creation(true);
        let normal_buffer = device.device().create_buffer(&normal_buffer_descriptor);
        let normal_array = js_sys::Float32Array::new(&normal_buffer.get_mapped_range());
        normal_array.copy_from(&normals);
        normal_buffer.unmap();
        // this
        let this = Self {
            index_count: indices.len(),
            position_buffer,
            index_buffer,
            normal_buffer,
            device: Arc::clone(device),
        };
        Arc::new(this)
    }

    pub fn index_count(&self) -> usize {
        self.index_count
    }

    pub fn position_buffer(&self) -> &GpuBuffer {
        &self.position_buffer
    }

    pub fn index_buffer(&self) -> &GpuBuffer {
        &self.index_buffer
    }

    pub fn normal_buffer(&self) -> &GpuBuffer {
        &self.normal_buffer
    }
}
