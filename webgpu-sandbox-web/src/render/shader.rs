
use std::sync::{Arc, Mutex};

use crate::{
    global,
};

use crate::render::device::Device;

use nalgebra_glm as glm;

// @see https://rustwasm.github.io/wasm-bindgen/examples/wasm-in-wasm.html
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::{spawn_local, JsFuture};

use web_sys::{
    Gpu,
    GpuAdapter,
    GpuDevice,
    GpuCanvasContext,
    GpuCanvasConfiguration,
    GpuTextureFormat,
    GpuShaderModule,
    GpuShaderModuleDescriptor,
    GpuPipelineLayoutDescriptor,
    GpuRenderPipelineDescriptor,
    GpuVertexState,
    GpuFragmentState,
    GpuCanvasAlphaMode,
    GpuColorTargetState,
    GpuPrimitiveState,
    GpuPrimitiveTopology,
    GpuRenderPassDescriptor,
    GpuRenderPassColorAttachment,
    GpuLoadOp,
    GpuStoreOp,
    GpuColorDict,
    GpuMultisampleState,
    GpuTextureDescriptor,
    gpu_texture_usage, 
    GpuBufferDescriptor,
    gpu_buffer_usage,
    GpuVertexBufferLayout,
    GpuVertexAttribute,
    GpuVertexFormat,
    GpuBindGroupDescriptor,
    GpuBindGroupEntry,
    GpuBindGroupLayoutEntry,
    gpu_shader_stage,
    GpuBufferBindingLayout,
    GpuBufferBinding,
    GpuBindGroupLayoutDescriptor,
    GpuBufferBindingType,
    GpuDepthStencilState,
    GpuCompareFunction,
    GpuRenderPassDepthStencilAttachment,
};

pub struct ShaderModule {
    device: Arc<Device>,
    shader_module: GpuShaderModule,
}

impl ShaderModule {
    pub fn with_code(device: &Arc<Device>, code: &str) -> Arc<Self> {
        let shader_descriptor = GpuShaderModuleDescriptor::new(code);
        let shader_module = device.device().create_shader_module(&shader_descriptor);
        Self::new(device, shader_module)
    }

    fn new(device: &Arc<Device>, shader_module: GpuShaderModule) -> Arc<Self> {
        let this = Self {
            device: Arc::clone(device),
            shader_module,
        };
        Arc::new(this)
    }

    pub fn shader_module(&self) -> &GpuShaderModule {
        &self.shader_module
    }
}
