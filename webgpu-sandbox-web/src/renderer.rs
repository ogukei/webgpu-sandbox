
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::asset::Mesh;
use crate::console_log;
use crate::render::{
    Device,
    Surface,
    Shaders,
    Vertices,
    SurfaceConfiguration,
    MeshBuffer,
};
use crate::render::PipelineLayouts;
use crate::scene::SceneContext;

use nalgebra_glm as glm;

use wasm_bindgen::prelude::*;

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
    GpuIndexFormat,
    GpuBindGroup,
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
        let mut depth_stencil_state = GpuDepthStencilState::new(GpuCompareFunction::Less, true, GpuTextureFormat::Depth24plus);
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
            render_pass_encoder.set_vertex_buffer(0, Some(vertex_buffer.buffer()));
            render_pass_encoder.set_bind_group(0, Some(&bind_group));
            render_pass_encoder.draw(vertex_buffer.vertex_count() as u32);
            render_pass_encoder.end();
            
            // write
            let queue = device.device().queue();
            let sky_height = 3.0;
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
        // model
        let model = scene_context.model();
        let mesh_buffers: HashMap<usize, Arc<MeshBuffer>>;
        mesh_buffers = model.meshes().iter()
            .map(|v| (v.mesh_index(), MeshBuffer::new(device, v.positions(), v.indices(), v.normals())))
            .collect();
        // shader
        let shader_module = Shaders::common(device);
        // vertex shader
        let mut vertex_state = GpuVertexState::new("vert_main", shader_module.shader_module());
        // layouts
        let vertex_buffer_layouts = {
            // attributes
            let positions_layout = {
                let vertex_buffer_attribute = GpuVertexAttribute::new(GpuVertexFormat::Float32x3, 0.0, 0);
                let vertex_buffer_attributes = vec![vertex_buffer_attribute];
                let vertex_buffer_attributes = vertex_buffer_attributes.into_iter().collect::<js_sys::Array>();
                let vertex_buffer_layout = GpuVertexBufferLayout::new(
                    (std::mem::size_of::<f32>() * 3) as f64,
                    &vertex_buffer_attributes);
                vertex_buffer_layout
            };
            let normals_layout = {
                let vertex_buffer_attribute = GpuVertexAttribute::new(GpuVertexFormat::Float32x3, 0.0, 1);
                let vertex_buffer_attributes = vec![vertex_buffer_attribute];
                let vertex_buffer_attributes = vertex_buffer_attributes.into_iter().collect::<js_sys::Array>();
                let vertex_buffer_layout = GpuVertexBufferLayout::new(
                    (std::mem::size_of::<f32>() * 3) as f64,
                    &vertex_buffer_attributes);
                vertex_buffer_layout
            };
            let vertex_buffer_layouts: Vec<JsValue> = vec![positions_layout.into(), normals_layout.into()];
            let vertex_buffer_layouts = vertex_buffer_layouts.into_iter().collect::<js_sys::Array>();
            vertex_buffer_layouts
        };
        vertex_state.buffers(&vertex_buffer_layouts);

        // init
        let layout = PipelineLayouts::common(device);
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
        let mut depth_stencil_state = GpuDepthStencilState::new(GpuCompareFunction::Less, true, GpuTextureFormat::Depth24plus);
        render_descriptor.depth_stencil(&depth_stencil_state);

        // render
        let render_pipeline = device.device().create_render_pipeline(&render_descriptor);

        // uniform
        let uniform_buffer_descriptor = GpuBufferDescriptor::new(
            (std::mem::size_of::<f32>() * (16 + 4)) as f64,
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
        // secondary bind group
        // entities
        let object_buffer_descriptor = GpuBufferDescriptor::new(
            (std::mem::size_of::<f32>() * 16 * 4 * model.nodes().len()) as f64,
            gpu_buffer_usage::UNIFORM | gpu_buffer_usage::COPY_DST);
        let object_buffer = device.device().create_buffer(&object_buffer_descriptor);
        let secondary_bind_groups: Vec<GpuBindGroup>;
        secondary_bind_groups = model.nodes().iter().enumerate()
            .map(|(local_index, _)| {
                let mut buffer_binding = GpuBufferBinding::new(&object_buffer);
                let chunk_size = std::mem::size_of::<f32>() * 16 * 4;
                buffer_binding.size(chunk_size as f64);
                buffer_binding.offset((chunk_size * local_index) as f64);
                let buffer_binding: JsValue = buffer_binding.into();
                let buffer_bind_entry = GpuBindGroupEntry::new(0, &buffer_binding);
                let bind_entries: Vec<JsValue> = vec![buffer_bind_entry.into()];
                let bind_entries = bind_entries.into_iter().collect::<js_sys::Array>();
                let bind_group_descriptor = GpuBindGroupDescriptor::new(&bind_entries, &render_pipeline.get_bind_group_layout(1));
                let bind_group = device.device().create_bind_group(&bind_group_descriptor);
                bind_group
            })
            .collect();
        // transfer
        {
            let queue = device.device().queue();
            let matrix_values: Vec<f32> = model.nodes().iter()
                .flat_map(|v| 
                    // align 256 bytes
                    vec![v.transform().as_slice(),
                         v.transform().as_slice(),
                         v.transform().as_slice(),
                         v.transform().as_slice()])
                .flatten()
                .copied()
                .collect();
            let object_data = js_sys::Float32Array::new_with_length(16 * 4 * model.nodes().len() as u32);
            object_data.copy_from(matrix_values.as_slice());
            queue.write_buffer_with_u32_and_buffer_source(&object_buffer, 0, &object_data);
        }
        // render
        let device = Arc::clone(device);
        let surface = Arc::clone(surface);
        let scene_context = Arc::clone(scene_context);
        let stage = Arc::clone(stage);
        let model = Arc::clone(model);
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
            render_pass_encoder.set_bind_group(0, Some(&bind_group));
            // nodes
            let nodes = model.nodes();
            for (local_index, node) in nodes.iter().enumerate() {
                let Some(mesh_index) = node.mesh_index() else { continue };
                let Some(mesh_buffer) = mesh_buffers.get(&mesh_index) else { continue };
                let Some(secondary_bind_group) = secondary_bind_groups.get(local_index) else { continue };
                render_pass_encoder.set_bind_group(1, Some(&secondary_bind_group));
                render_pass_encoder.set_vertex_buffer(0, Some(mesh_buffer.position_buffer()));
                render_pass_encoder.set_vertex_buffer(1, Some(mesh_buffer.normal_buffer()));
                render_pass_encoder.set_index_buffer(mesh_buffer.index_buffer(), GpuIndexFormat::Uint32);
                render_pass_encoder.draw_indexed(mesh_buffer.index_count() as u32);
            }
            render_pass_encoder.end();
            
            // write
            let queue = device.device().queue();
            {
                let size = std::mem::size_of::<f32>() * (16 + 4);
                let projection_view_matrix = stage.projection_view_matrix(&surface_configuration, &scene_context);
                let camera_position = stage.camera_position(&surface_configuration, &scene_context);
                let uniform_array = js_sys::Uint8Array::new_with_length(size as u32);
                let uniform_data = UniformData {
                    projection_view_matrix,
                    camera_position,
                };
                let ptr = (&uniform_data as *const _) as *const u8;
                let slice = unsafe {
                    std::slice::from_raw_parts(ptr, size)
                };
                uniform_array.copy_from(slice);
                queue.write_buffer_with_u32_and_buffer_source(&uniform_buffer, 0, &uniform_array);
            }
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
    
    fn view_matrix(&self, surface_configuration: &SurfaceConfiguration, scene_context: &Arc<SceneContext>) -> glm::Mat4 {
        let model = scene_context.model();
        let center: glm::Vec3;
        let eye: glm::Vec3;
        if let Some(bounding_box) = model.bounding_box() {
            center = (bounding_box.max() + bounding_box.min()) * 0.5;
            let v = bounding_box.min();
            eye = glm::vec3(center.x, (center.y + bounding_box.max().y) * 0.5, v.z * 3.0);
        } else {
            eye = glm::vec3(0.25, 0.0, -0.0);
            center = glm::vec3(0.0,  0.0, 0.0);
        }
        let eye = glm::quat_to_mat4(&scene_context.view_quat()) * glm::vec4(eye.x, eye.y, eye.z, 1.0);
        let eye = eye.xyz();
        let look_at_default = glm::look_at(&eye, &center, &glm::vec3(0.0,  1.0, 0.0));
        let view_matrix = look_at_default;
        view_matrix
    }

    pub fn projection_view_matrix(&self, surface_configuration: &SurfaceConfiguration, scene_context: &Arc<SceneContext>) -> glm::Mat4 {
        let aspect = (surface_configuration.width() as f64 / surface_configuration.height() as f64) as f32;
        let fovy: f32 = 90.0;
        let fovy = fovy.to_radians();
        let projection_matrix = glm::perspective(aspect, fovy, 0.001, 100.0);
        let view_matrix = self.view_matrix(surface_configuration, scene_context);
        projection_matrix * view_matrix
    }

    fn camera_position(&self, surface_configuration: &SurfaceConfiguration, scene_context: &Arc<SceneContext>) -> glm::Vec4 {
        let view_matrix = self.view_matrix(surface_configuration, scene_context);
        let inverse_view_matrix = view_matrix.try_inverse().unwrap_or_else(|| glm::identity());
        inverse_view_matrix * glm::vec4(0.0, 0.0, 0.0, 1.0)
    }
}

#[repr(C)]
struct UniformData {
    pub projection_view_matrix: glm::Mat4,
    pub camera_position: glm::Vec4,
}
