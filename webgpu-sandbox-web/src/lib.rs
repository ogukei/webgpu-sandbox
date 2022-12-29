
mod console;
mod global;

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
    GpuBindGroup,
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

    let presentation_format = gpu.get_preferred_canvas_format();
    let mut canvas_configuration = GpuCanvasConfiguration::new(&device, presentation_format);
    canvas_configuration.alpha_mode(GpuCanvasAlphaMode::Opaque);
    context.configure(&canvas_configuration);
    console_log!("context configured");

    // https://www.w3.org/TR/WGSL/#example-b218a1e2
    // https://github.com/austinEng/webgpu-samples/blob/main/src/sample/helloTriangle/main.ts
    let code = "
@vertex
fn vert_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    var vertices = array<vec2<f32>, 3>(
        vec2(0.0, 0.5),
        vec2(-0.5, -0.5),
        vec2(0.5, -0.5)
    );
    var vert = vertices[index];
    return vec4<f32>(vert.x, vert.y, 0.0, 1.0);
}

@fragment
fn frag_main(@builtin(position) coord_in: vec4<f32>) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
";
    let shader_descriptor = GpuShaderModuleDescriptor::new(code);
    let shader_module = device.create_shader_module(&shader_descriptor);
    console_log!("shader module acquired");

    // layout
    let binds: Vec<JsValue> = vec![];
    let binds = binds.into_iter().collect::<js_sys::Array>();
    let layout_descriptor = GpuPipelineLayoutDescriptor::new(&binds);
    let layout = device.create_pipeline_layout(&layout_descriptor);
    
    // vertex
    let vertex_state = GpuVertexState::new("vert_main", &shader_module);
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

    // render
    let render_pipeline = device.create_render_pipeline(&render_descriptor);

    // frame
    let command_encoder = device.create_command_encoder();
    let texture = context.get_current_texture();
    let texture_view = texture.create_view();

    // render pass
    let mut color_attachment = GpuRenderPassColorAttachment::new(GpuLoadOp::Clear, GpuStoreOp::Store, &texture_view);
    let clear_color = GpuColorDict::new(1.0, 0.0, 0.0, 0.0);
    let clear_color: JsValue = clear_color.into();
    color_attachment.clear_value(&clear_color);
    let color_attachments: Vec<JsValue> = vec![
        color_attachment.into(),
    ];
    let color_attachments = color_attachments.into_iter().collect::<js_sys::Array>();
    let render_pass_descriptor = GpuRenderPassDescriptor::new(&color_attachments);
    
    // render pass encoder
    let render_pass_encoder = command_encoder.begin_render_pass(&render_pass_descriptor);
    render_pass_encoder.set_pipeline(&render_pipeline);
    render_pass_encoder.draw(3);
    render_pass_encoder.end();

    // submit
    let command_buffer = command_encoder.finish();
    let queue = device.queue();
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
