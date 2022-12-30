
mod console;
mod global;

use nalgebra_glm as glm;

// @see https://rustwasm.github.io/wasm-bindgen/examples/wasm-in-wasm.html
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::{spawn_local, JsFuture};

use web_sys::{
    GpuAdapter,
    GpuDevice,
    GpuCanvasContext,
    GpuCanvasConfiguration,
    GpuTextureFormat,
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

async fn main() -> Result<(), JsValue> {
    console_log!("initializing...");
    let gpu = global::gpu();
    let adapter = JsFuture::from(gpu.request_adapter()).await?;
    let adapter: GpuAdapter = adapter.unchecked_into();
    let device = JsFuture::from(adapter.request_device()).await?;
    let device: GpuDevice = device.unchecked_into();
    console_log!("device acquired");

    let window = global::window();
    let document = window.document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.unchecked_into();
    let context = canvas.get_context("webgpu").unwrap().unwrap();
    let context: GpuCanvasContext = context.unchecked_into();
    console_log!("context acquired");
    // size 
    let device_pixel_ratio = window.device_pixel_ratio();
    let width = canvas.client_width() as f64 * device_pixel_ratio;
    let height = canvas.client_height() as f64 * device_pixel_ratio;
    let presentation_size: Vec<JsValue> = vec![width, height].into_iter().map(Into::into).collect();
    let presentation_size = presentation_size.into_iter().collect::<js_sys::Array>();
    // configure canvas size
    canvas.set_width(width as u32);
    canvas.set_height(height as u32);
    // format
    let presentation_format = gpu.get_preferred_canvas_format();
    let mut canvas_configuration = GpuCanvasConfiguration::new(&device, presentation_format);
    canvas_configuration.alpha_mode(GpuCanvasAlphaMode::Opaque);
    context.configure(&canvas_configuration);
    console_log!("context configured");

    // https://www.w3.org/TR/WGSL/#example-b218a1e2
    // https://github.com/austinEng/webgpu-samples/blob/main/src/sample/helloTriangle/main.ts
    let code = "

struct Uniforms {
    projection_view: mat4x4<f32>,
}
@binding(0) @group(0) var<uniform> uniforms: Uniforms;

@vertex
fn vert_main(
    @builtin(vertex_index) index: u32,
    @location(0) position: vec3<f32>
) -> @builtin(position) vec4<f32> {
    var p = vec4<f32>(position.x, position.y, position.z, 1.0);
    return uniforms.projection_view * p;
}

@fragment
fn frag_main(@builtin(position) coord_in: vec4<f32>) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
";
    let shader_descriptor = GpuShaderModuleDescriptor::new(code);
    let shader_module = device.create_shader_module(&shader_descriptor);
    console_log!("shader module acquired");

    // vertices
    let vertices: Vec<f32> = vec![
        -0.5, -0.5,  -0.5,
        -0.5,  0.5,  -0.5,
         0.5, -0.5,  -0.5,
        -0.5,  0.5,  -0.5,
         0.5,  0.5,  -0.5,
         0.5, -0.5,  -0.5,
    
        -0.5, -0.5,   0.5,
         0.5, -0.5,   0.5,
        -0.5,  0.5,   0.5,
        -0.5,  0.5,   0.5,
         0.5, -0.5,   0.5,
         0.5,  0.5,   0.5,
    
        -0.5,   0.5, -0.5,
        -0.5,   0.5,  0.5,
         0.5,   0.5, -0.5,
        -0.5,   0.5,  0.5,
         0.5,   0.5,  0.5,
         0.5,   0.5, -0.5,
    
        -0.5,  -0.5, -0.5,
         0.5,  -0.5, -0.5,
        -0.5,  -0.5,  0.5,
        -0.5,  -0.5,  0.5,
         0.5,  -0.5, -0.5,
         0.5,  -0.5,  0.5,
    
        -0.5,  -0.5, -0.5,
        -0.5,  -0.5,  0.5,
        -0.5,   0.5, -0.5,
        -0.5,  -0.5,  0.5,
        -0.5,   0.5,  0.5,
        -0.5,   0.5, -0.5,
    
         0.5,  -0.5, -0.5,
         0.5,   0.5, -0.5,
         0.5,  -0.5,  0.5,
         0.5,  -0.5,  0.5,
         0.5,   0.5, -0.5,
         0.5,   0.5,  0.5,
    ];
    let vertex_count = vertices.len() / 3;
    let mut vertex_buffer_descriptor = GpuBufferDescriptor::new(
        (std::mem::size_of::<f32>() * vertices.len()) as f64,
        gpu_buffer_usage::VERTEX);
    vertex_buffer_descriptor.mapped_at_creation(true);
    let vertex_buffer = device.create_buffer(&vertex_buffer_descriptor);
    let vertex_array = js_sys::Float32Array::new(&vertex_buffer.get_mapped_range());
    vertex_array.copy_from(&vertices);
    vertex_buffer.unmap();

    // layout
    let mut bind_group_layout_entry = GpuBindGroupLayoutEntry::new(0, gpu_shader_stage::VERTEX);
    let mut buffer_bind_group_layout_entry = GpuBufferBindingLayout::new();
    buffer_bind_group_layout_entry.type_(GpuBufferBindingType::Uniform);
    bind_group_layout_entry.buffer(&buffer_bind_group_layout_entry);
    let bind_group_layout_entries: Vec<JsValue> = vec![bind_group_layout_entry.into()];
    let bind_group_layout_entries = bind_group_layout_entries.into_iter().collect::<js_sys::Array>();
    let bind_group_layout_descriptor = GpuBindGroupLayoutDescriptor::new(&bind_group_layout_entries);
    let bind_group_layout = device.create_bind_group_layout(&bind_group_layout_descriptor);
    let bind_group_layouts: Vec<JsValue> = vec![bind_group_layout.into()];
    let bind_group_layouts = bind_group_layouts.into_iter().collect::<js_sys::Array>();
    let layout_descriptor = GpuPipelineLayoutDescriptor::new(&bind_group_layouts);
    let layout = device.create_pipeline_layout(&layout_descriptor);
    
    // vertex
    let mut vertex_state = GpuVertexState::new("vert_main", &shader_module);
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
    let mut render_descriptor = GpuRenderPipelineDescriptor::new(&layout, &vertex_state);
    // fragment
    let target = GpuColorTargetState::new(presentation_format);
    let fragment_targets: Vec<JsValue> = vec![target.into()];
    let fragment_targets = fragment_targets.into_iter().collect::<js_sys::Array>();
    let fragment_state = GpuFragmentState::new("frag_main", &shader_module, &fragment_targets);
    render_descriptor.fragment(&fragment_state);
    // primitive
    let mut primitive_state = GpuPrimitiveState::new();
    primitive_state.topology(GpuPrimitiveTopology::TriangleList);
    render_descriptor.primitive(&primitive_state);

    // multisample
    let sample_count = 4;
    let mut multisample_state = GpuMultisampleState::new();
    multisample_state.count(sample_count);
    render_descriptor.multisample(&multisample_state);

    // depth stencil
    let mut depth_stencil_state = GpuDepthStencilState::new(GpuTextureFormat::Depth24plus);
    depth_stencil_state.depth_write_enabled(true);
    depth_stencil_state.depth_compare(GpuCompareFunction::Less);
    render_descriptor.depth_stencil(&depth_stencil_state);

    // render
    let render_pipeline = device.create_render_pipeline(&render_descriptor);

    // uniform
    let uniform_buffer_descriptor = GpuBufferDescriptor::new(
        (std::mem::size_of::<f32>() * 16) as f64,
        gpu_buffer_usage::UNIFORM | gpu_buffer_usage::COPY_DST);
    let uniform_buffer = device.create_buffer(&uniform_buffer_descriptor);
    // entries
    // https://www.w3.org/TR/webgpu/#dictdef-gpubindgroupdescriptor
    let buffer_binding = GpuBufferBinding::new(&uniform_buffer);
    let buffer_binding: JsValue = buffer_binding.into();
    let buffer_bind_entry = GpuBindGroupEntry::new(0, &buffer_binding);
    let bind_entries: Vec<JsValue> = vec![buffer_bind_entry.into()];
    let bind_entries = bind_entries.into_iter().collect::<js_sys::Array>();
    let bind_group_descriptor = GpuBindGroupDescriptor::new(&bind_entries, &render_pipeline.get_bind_group_layout(0));
    let bind_group = device.create_bind_group(&bind_group_descriptor);

    // texture
    let mut texture_descriptor = GpuTextureDescriptor::new(
        presentation_format, &presentation_size, gpu_texture_usage::RENDER_ATTACHMENT);
    texture_descriptor.sample_count(sample_count);
    let texture = device.create_texture(&texture_descriptor);
    let texture_view = texture.create_view();

    // depth texture
    let mut depth_texture_descriptor = GpuTextureDescriptor::new(
        GpuTextureFormat::Depth24plus, &presentation_size, gpu_texture_usage::RENDER_ATTACHMENT);
    depth_texture_descriptor.sample_count(sample_count);
    let depth_texture = device.create_texture(&depth_texture_descriptor);

    // frame
    let command_encoder = device.create_command_encoder();
    let context_texture_view = context.get_current_texture().create_view();

    // render pass
    let mut color_attachment = GpuRenderPassColorAttachment::new(
        GpuLoadOp::Clear, GpuStoreOp::Discard, &texture_view);
    let clear_color = GpuColorDict::new(1.0, 0.0, 0.0, 0.0);
    let clear_color: JsValue = clear_color.into();
    color_attachment.clear_value(&clear_color);
    color_attachment.resolve_target(&context_texture_view);
    let color_attachments: Vec<JsValue> = vec![
        color_attachment.into(),
    ];
    let color_attachments = color_attachments.into_iter().collect::<js_sys::Array>();
    let mut render_pass_descriptor = GpuRenderPassDescriptor::new(&color_attachments);
    // depth stencil
    let mut depth_stencil_attachment = GpuRenderPassDepthStencilAttachment::new(&depth_texture.create_view());
    depth_stencil_attachment.depth_load_op(GpuLoadOp::Clear);
    depth_stencil_attachment.depth_store_op(GpuStoreOp::Store);
    depth_stencil_attachment.depth_clear_value(1.0);
    render_pass_descriptor.depth_stencil_attachment(&depth_stencil_attachment);
    // render pass encoder
    let render_pass_encoder = command_encoder.begin_render_pass(&render_pass_descriptor);
    render_pass_encoder.set_pipeline(&render_pipeline);
    render_pass_encoder.set_vertex_buffer(0, &vertex_buffer);
    render_pass_encoder.set_bind_group(0, &bind_group);
    render_pass_encoder.draw(vertex_count as u32);
    render_pass_encoder.end();
    
    // write
    let queue = device.queue();
    let aspect = (width as f64 / height as f64) as f32;
    let fovy: f32 = 90.0;
    let fovy = fovy.to_radians();
    let perspective_matrix = glm::perspective(aspect, fovy, 0.01, 10.0);
    let view_matrix = glm::translation(&glm::vec3(0.0,  0.0, -1.5));
    let projection_view_matrix = perspective_matrix * view_matrix;
    let uniform_data = js_sys::Float32Array::new_with_length(16);
    uniform_data.copy_from(projection_view_matrix.as_slice());
    queue.write_buffer_with_u32_and_buffer_source(&uniform_buffer, 0, &uniform_data);

    // submit
    let command_buffer = command_encoder.finish();
    let command_buffers:  Vec<JsValue> = vec![
        command_buffer.into(),
    ];
    let command_buffers = command_buffers.into_iter().collect::<js_sys::Array>();
    queue.submit(&command_buffers);
    console_log!("draw");
    Ok(())
}

#[wasm_bindgen(start)]
pub fn run() {
    spawn_local(async {
        main().await
            .unwrap_throw();
    });
}
