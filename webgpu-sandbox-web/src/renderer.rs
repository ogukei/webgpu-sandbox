
use std::sync::{Arc, Mutex};

use crate::render::{
    Device,
    Surface,
    Shaders,
    Vertices, SurfaceConfiguration,
};
use crate::render::PipelineLayouts;
use crate::scene::{SceneContext, self};

use nalgebra_glm as glm;

use wasm_bindgen::{prelude::*};

use web_sys::{
    GpuTextureFormat,
    GpuRenderPipelineDescriptor,
    GpuVertexState,
    GpuFragmentState,
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
    GpuBufferBinding,
    GpuDepthStencilState,
    GpuCompareFunction,
    GpuRenderPassDepthStencilAttachment,
    GpuTexture,
};

pub struct Renderer {
    render_frame: Box<dyn Fn() + 'static>,
}

impl Renderer {
    pub fn new(device: &Arc<Device>, surface: &Arc<Surface>, scene_context: &Arc<SceneContext>) -> Arc<Self> {
        let stage = RenderStage::new(device, surface);
        let skybox_render_pipeline = SkyboxRenderPipeline::new(device, surface, scene_context, &stage);
        let final_render_pipeline = FinalRenderPipeline::new(device, surface, scene_context, &stage);
        let render_frame = move || {
            skybox_render_pipeline.render_frame();
            final_render_pipeline.render_frame();
        };
        let this = Self {
            render_frame: Box::new(render_frame),
        };
        Arc::new(this)
    }

    pub fn render_frame(&self) {
        let func = self.render_frame.as_ref();
        func();
    }
}

struct SkyboxRenderPipeline {
    render_frame: Box<dyn Fn() + 'static>,
}

impl SkyboxRenderPipeline {
    pub fn new(device: &Arc<Device>,
        surface: &Arc<Surface>,
        scene_context: &Arc<SceneContext>,
        stage: &Arc<RenderStage>) -> Arc<Self> {
        let surface_configuration = surface.configuration();
        let presentation_format = surface_configuration.presentation_format();
        // skybox cube
        let vertex_buffer = Vertices::cube(device);
        let shader_module = Shaders::skybox(device);
        // vertex shader
        let mut vertex_state = GpuVertexState::new("vert_main", shader_module.shader_module());
        // attributes
        let vertex_buffer_attribute = GpuVertexAttribute::new(GpuVertexFormat::Float32x3, 0.0, 0);
        let vertex_buffer_attributes = vec![vertex_buffer_attribute];
        let vertex_buffer_attributes = vertex_buffer_attributes.into_iter().collect::<js_sys::Array>();
        // layouts
        let vertex_buffer_layout = GpuVertexBufferLayout::new(
            (std::mem::size_of::<f32>() * 3) as f64,
            &vertex_buffer_attributes);
        let vertex_buffer_layouts: Vec<JsValue> = vec![vertex_buffer_layout.into()];
        let vertex_buffer_layouts = vertex_buffer_layouts.into_iter().collect::<js_sys::Array>();
        vertex_state.buffers(&vertex_buffer_layouts);

        // init
        let layout = PipelineLayouts::skybox(device);
        let mut render_descriptor = GpuRenderPipelineDescriptor::new(layout.layout(), &vertex_state);
        // fragment
        let target = GpuColorTargetState::new(presentation_format);
        let fragment_targets: Vec<JsValue> = vec![target.into()];
        let fragment_targets = fragment_targets.into_iter().collect::<js_sys::Array>();
        let fragment_state = GpuFragmentState::new("frag_main", shader_module.shader_module(), &fragment_targets);
        render_descriptor.fragment(&fragment_state);
        // primitive
        let mut primitive_state = GpuPrimitiveState::new();
        primitive_state.topology(GpuPrimitiveTopology::TriangleList);
        render_descriptor.primitive(&primitive_state);

        // multisample
        let sample_count = stage.sample_count();
        let mut multisample_state = GpuMultisampleState::new();
        multisample_state.count(sample_count);
        render_descriptor.multisample(&multisample_state);

        // depth stencil
        let mut depth_stencil_state = GpuDepthStencilState::new(GpuTextureFormat::Depth24plus);
        depth_stencil_state.depth_write_enabled(true);
        depth_stencil_state.depth_compare(GpuCompareFunction::Less);
        render_descriptor.depth_stencil(&depth_stencil_state);

        // render
        let render_pipeline = device.device().create_render_pipeline(&render_descriptor);

        // uniform
        let uniform_buffer_descriptor = GpuBufferDescriptor::new(
            (std::mem::size_of::<f32>() * 16) as f64,
            gpu_buffer_usage::UNIFORM | gpu_buffer_usage::COPY_DST);
        let uniform_buffer = device.device().create_buffer(&uniform_buffer_descriptor);
        // entries
        // https://www.w3.org/TR/webgpu/#dictdef-gpubindgroupdescriptor
        let buffer_binding = GpuBufferBinding::new(&uniform_buffer);
        let buffer_binding: JsValue = buffer_binding.into();
        let buffer_bind_entry = GpuBindGroupEntry::new(0, &buffer_binding);
        let bind_entries: Vec<JsValue> = vec![buffer_bind_entry.into()];
        let bind_entries = bind_entries.into_iter().collect::<js_sys::Array>();
        let bind_group_descriptor = GpuBindGroupDescriptor::new(&bind_entries, &render_pipeline.get_bind_group_layout(0));
        let bind_group = device.device().create_bind_group(&bind_group_descriptor);

        // render
        let device = Arc::clone(device);
        let scene_context = Arc::clone(scene_context);
        let stage = Arc::clone(stage);
        let render_frame = move || {
            // frame
            let command_encoder = device.device().create_command_encoder();
            // render pass
            let mut color_attachment = GpuRenderPassColorAttachment::new(
                GpuLoadOp::Clear, GpuStoreOp::Store, &stage.color_texture().create_view());
            let clear_color = GpuColorDict::new(1.0, 0.0, 0.0, 0.0);
            let clear_color: JsValue = clear_color.into();
            color_attachment.clear_value(&clear_color);
            let color_attachments: Vec<JsValue> = vec![
                color_attachment.into(),
            ];
            let color_attachments = color_attachments.into_iter().collect::<js_sys::Array>();
            let mut render_pass_descriptor = GpuRenderPassDescriptor::new(&color_attachments);
            // depth stencil
            let mut depth_stencil_attachment = GpuRenderPassDepthStencilAttachment::new(&stage.depth_texture().create_view());
            depth_stencil_attachment.depth_load_op(GpuLoadOp::Clear);
            depth_stencil_attachment.depth_store_op(GpuStoreOp::Store);
            depth_stencil_attachment.depth_clear_value(1.0);
            render_pass_descriptor.depth_stencil_attachment(&depth_stencil_attachment);

            // render pass encoder
            let render_pass_encoder = command_encoder.begin_render_pass(&render_pass_descriptor);
            render_pass_encoder.set_pipeline(&render_pipeline);
            render_pass_encoder.set_vertex_buffer(0, vertex_buffer.buffer());
            render_pass_encoder.set_bind_group(0, &bind_group);
            render_pass_encoder.draw(vertex_buffer.vertex_count() as u32);
            render_pass_encoder.end();
            
            // write
            let queue = device.device().queue();
            let sky_height = 5.0;
            let projection_view_matrix = stage.projection_view_matrix(&surface_configuration, &scene_context);
            let model_matrix = glm::scaling(&glm::vec3(sky_height * 2.0, sky_height * 2.0, sky_height * 2.0));
            let projection_view_model = projection_view_matrix * model_matrix;
            let uniform_data = js_sys::Float32Array::new_with_length(16);
            uniform_data.copy_from(projection_view_model.as_slice());
            queue.write_buffer_with_u32_and_buffer_source(&uniform_buffer, 0, &uniform_data);

            // submit
            let command_buffer = command_encoder.finish();
            let command_buffers: Vec<JsValue> = vec![
                command_buffer.into(),
            ];
            let command_buffers = command_buffers.into_iter().collect::<js_sys::Array>();
            queue.submit(&command_buffers);
        };
        let this = Self {
            render_frame: Box::new(render_frame),
        };
        Arc::new(this)
    }

    pub fn render_frame(&self) {
        let func = self.render_frame.as_ref();
        func();
    }
}


struct FinalRenderPipeline {
    render_frame: Box<dyn Fn() + 'static>,
}

impl FinalRenderPipeline {
    pub fn new(device: &Arc<Device>,
        surface: &Arc<Surface>,
        scene_context: &Arc<SceneContext>,
        stage: &Arc<RenderStage>) -> Arc<Self> {
        let surface_configuration = surface.configuration();
        let presentation_format = surface_configuration.presentation_format();
        // skybox cube
        let vertex_buffer = Vertices::cube(device);
        let shader_module = Shaders::cube(device);
        // vertex shader
        let mut vertex_state = GpuVertexState::new("vert_main", shader_module.shader_module());
        // attributes
        let vertex_buffer_attribute = GpuVertexAttribute::new(GpuVertexFormat::Float32x3, 0.0, 0);
        let vertex_buffer_attributes = vec![vertex_buffer_attribute];
        let vertex_buffer_attributes = vertex_buffer_attributes.into_iter().collect::<js_sys::Array>();
        // layouts
        let vertex_buffer_layout = GpuVertexBufferLayout::new(
            (std::mem::size_of::<f32>() * 3) as f64,
            &vertex_buffer_attributes);
        let vertex_buffer_layouts: Vec<JsValue> = vec![vertex_buffer_layout.into()];
        let vertex_buffer_layouts = vertex_buffer_layouts.into_iter().collect::<js_sys::Array>();
        vertex_state.buffers(&vertex_buffer_layouts);

        // init
        let layout = PipelineLayouts::skybox(device);
        let mut render_descriptor = GpuRenderPipelineDescriptor::new(layout.layout(), &vertex_state);
        // fragment
        let target = GpuColorTargetState::new(presentation_format);
        let fragment_targets: Vec<JsValue> = vec![target.into()];
        let fragment_targets = fragment_targets.into_iter().collect::<js_sys::Array>();
        let fragment_state = GpuFragmentState::new("frag_main", shader_module.shader_module(), &fragment_targets);
        render_descriptor.fragment(&fragment_state);
        // primitive
        let mut primitive_state = GpuPrimitiveState::new();
        primitive_state.topology(GpuPrimitiveTopology::TriangleList);
        render_descriptor.primitive(&primitive_state);

        // multisample
        let sample_count = stage.sample_count();
        let mut multisample_state = GpuMultisampleState::new();
        multisample_state.count(sample_count);
        render_descriptor.multisample(&multisample_state);

        // depth stencil
        let mut depth_stencil_state = GpuDepthStencilState::new(GpuTextureFormat::Depth24plus);
        depth_stencil_state.depth_write_enabled(true);
        depth_stencil_state.depth_compare(GpuCompareFunction::Less);
        render_descriptor.depth_stencil(&depth_stencil_state);

        // render
        let render_pipeline = device.device().create_render_pipeline(&render_descriptor);

        // uniform
        let uniform_buffer_descriptor = GpuBufferDescriptor::new(
            (std::mem::size_of::<f32>() * 16) as f64,
            gpu_buffer_usage::UNIFORM | gpu_buffer_usage::COPY_DST);
        let uniform_buffer = device.device().create_buffer(&uniform_buffer_descriptor);
        // entries
        // https://www.w3.org/TR/webgpu/#dictdef-gpubindgroupdescriptor
        let buffer_binding = GpuBufferBinding::new(&uniform_buffer);
        let buffer_binding: JsValue = buffer_binding.into();
        let buffer_bind_entry = GpuBindGroupEntry::new(0, &buffer_binding);
        let bind_entries: Vec<JsValue> = vec![buffer_bind_entry.into()];
        let bind_entries = bind_entries.into_iter().collect::<js_sys::Array>();
        let bind_group_descriptor = GpuBindGroupDescriptor::new(&bind_entries, &render_pipeline.get_bind_group_layout(0));
        let bind_group = device.device().create_bind_group(&bind_group_descriptor);

        // render
        let device = Arc::clone(device);
        let surface = Arc::clone(surface);
        let scene_context = Arc::clone(scene_context);
        let stage = Arc::clone(stage);
        let render_frame = move || {
            // frame
            let command_encoder = device.device().create_command_encoder();
            let context_texture_view = surface.canvas_context().get_current_texture().create_view();

            // render pass
            let mut color_attachment = GpuRenderPassColorAttachment::new(
                GpuLoadOp::Load, GpuStoreOp::Discard, &stage.color_texture().create_view());
            color_attachment.resolve_target(&context_texture_view);
            let color_attachments: Vec<JsValue> = vec![
                color_attachment.into(),
            ];
            let color_attachments = color_attachments.into_iter().collect::<js_sys::Array>();
            let mut render_pass_descriptor = GpuRenderPassDescriptor::new(&color_attachments);
            // depth stencil
            let mut depth_stencil_attachment = GpuRenderPassDepthStencilAttachment::new(&stage.depth_texture().create_view());
            depth_stencil_attachment.depth_load_op(GpuLoadOp::Clear);
            depth_stencil_attachment.depth_store_op(GpuStoreOp::Store);
            depth_stencil_attachment.depth_clear_value(1.0);
            render_pass_descriptor.depth_stencil_attachment(&depth_stencil_attachment);

            // render pass encoder
            let render_pass_encoder = command_encoder.begin_render_pass(&render_pass_descriptor);
            render_pass_encoder.set_pipeline(&render_pipeline);
            render_pass_encoder.set_vertex_buffer(0, vertex_buffer.buffer());
            render_pass_encoder.set_bind_group(0, &bind_group);
            render_pass_encoder.draw(vertex_buffer.vertex_count() as u32);
            render_pass_encoder.end();
            
            // write
            let queue = device.device().queue();
            let projection_view_matrix = stage.projection_view_matrix(&surface_configuration, &scene_context);
            let uniform_data = js_sys::Float32Array::new_with_length(16);
            uniform_data.copy_from(projection_view_matrix.as_slice());
            queue.write_buffer_with_u32_and_buffer_source(&uniform_buffer, 0, &uniform_data);

            // submit
            let command_buffer = command_encoder.finish();
            let command_buffers: Vec<JsValue> = vec![
                command_buffer.into(),
            ];
            let command_buffers = command_buffers.into_iter().collect::<js_sys::Array>();
            queue.submit(&command_buffers);
        };
        let this = Self {
            render_frame: Box::new(render_frame),
        };
        Arc::new(this)
    }

    pub fn render_frame(&self) {
        let func = self.render_frame.as_ref();
        func();
    }
}

struct RenderStage {
    color_texture: GpuTexture,
    depth_texture: GpuTexture,
    sample_count: u32,
}

impl RenderStage {
    pub fn new(device: &Arc<Device>, surface: &Arc<Surface>) -> Arc<Self> {
        let sample_count = 4;
        let surface_configuration = surface.configuration();
        let presentation_format = surface_configuration.presentation_format();
        let presentation_size = surface_configuration.presentation_size();
        // color texture
        let mut color_texture_descriptor = GpuTextureDescriptor::new(
            presentation_format, &presentation_size, gpu_texture_usage::RENDER_ATTACHMENT);
        color_texture_descriptor.sample_count(sample_count);
        let color_texture = device.device().create_texture(&color_texture_descriptor);
        // depth texture
        let mut depth_texture_descriptor = GpuTextureDescriptor::new(
            GpuTextureFormat::Depth24plus, &presentation_size, gpu_texture_usage::RENDER_ATTACHMENT);
        depth_texture_descriptor.sample_count(sample_count);
        let depth_texture = device.device().create_texture(&depth_texture_descriptor);
        // this
        let this = Self {
            color_texture,
            depth_texture,
            sample_count,
        };
        Arc::new(this)
    }

    pub fn color_texture(&self) -> &GpuTexture {
        &self.color_texture
    }

    pub fn depth_texture(&self) -> &GpuTexture {
        &self.depth_texture
    }

    pub fn sample_count(&self) -> u32 {
        self.sample_count
    }

    pub fn projection_view_matrix(&self, surface_configuration: &SurfaceConfiguration, scene_context: &Arc<SceneContext>) -> glm::Mat4 {
        let aspect = (surface_configuration.width() as f64 / surface_configuration.height() as f64) as f32;
        let fovy: f32 = 90.0;
        let fovy = fovy.to_radians();
        let projection_matrix = glm::perspective(aspect, fovy, 0.001, 10.0);
        let look_at_default = glm::look_at(&glm::vec3(0.25,  1.0, -1.5), &glm::vec3(0.0,  0.0, 0.0), &glm::vec3(0.0,  1.0, 0.0));
        let view_matrix = look_at_default *  glm::quat_to_mat4(&scene_context.view_quat());
        projection_matrix * view_matrix
    }
}
