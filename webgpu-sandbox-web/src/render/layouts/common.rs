

use std::sync::Arc;

use super::PipelineLayouts;
use crate::render::device::Device;
use crate::render::layout::PipelineLayout;

use wasm_bindgen::{prelude::*};

use web_sys::{
    GpuPipelineLayoutDescriptor,
    GpuBindGroupLayoutEntry,
    gpu_shader_stage,
    GpuBufferBindingLayout,
    GpuBindGroupLayoutDescriptor,
    GpuBufferBindingType,
};

impl PipelineLayouts {
    pub fn common(device: &Arc<Device>) -> Arc<PipelineLayout> {
        // primary
        let primary_layout = {
            let mut bind_group_layout_entry = GpuBindGroupLayoutEntry::new(0, gpu_shader_stage::VERTEX);
            let mut buffer_bind_group_layout_entry = GpuBufferBindingLayout::new();
            buffer_bind_group_layout_entry.type_(GpuBufferBindingType::Uniform);
            bind_group_layout_entry.buffer(&buffer_bind_group_layout_entry);
            let bind_group_layout_entries: Vec<JsValue> = vec![bind_group_layout_entry.into()];
            let bind_group_layout_entries = bind_group_layout_entries.into_iter().collect::<js_sys::Array>();
            let bind_group_layout_descriptor = GpuBindGroupLayoutDescriptor::new(&bind_group_layout_entries);
            let bind_group_layout = device.device().create_bind_group_layout(&bind_group_layout_descriptor);
            bind_group_layout
        };
        // secondary 
        let secondary_layout = {
            let mut bind_group_layout_entry = GpuBindGroupLayoutEntry::new(0, gpu_shader_stage::VERTEX);
            let mut buffer_bind_group_layout_entry = GpuBufferBindingLayout::new();
            buffer_bind_group_layout_entry.type_(GpuBufferBindingType::Uniform);
            bind_group_layout_entry.buffer(&buffer_bind_group_layout_entry);
            let bind_group_layout_entries: Vec<JsValue> = vec![bind_group_layout_entry.into()];
            let bind_group_layout_entries = bind_group_layout_entries.into_iter().collect::<js_sys::Array>();
            let bind_group_layout_descriptor = GpuBindGroupLayoutDescriptor::new(&bind_group_layout_entries);
            let bind_group_layout = device.device().create_bind_group_layout(&bind_group_layout_descriptor);
            bind_group_layout
        };
        let bind_group_layouts: Vec<JsValue> = vec![primary_layout.into(), secondary_layout.into()];
        let bind_group_layouts = bind_group_layouts.into_iter().collect::<js_sys::Array>();
        let layout_descriptor = GpuPipelineLayoutDescriptor::new(&bind_group_layouts);
        let layout = device.device().create_pipeline_layout(&layout_descriptor);
        PipelineLayout::new(device, layout)
    }
}
