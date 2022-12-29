
mod console;
mod global;

// @see https://rustwasm.github.io/wasm-bindgen/examples/wasm-in-wasm.html
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::{spawn_local, JsFuture};

use web_sys::{GpuAdapter, GpuDevice, GpuCanvasContext, GpuCanvasConfiguration, GpuTextureFormat};

async fn run_async() -> Result<(), JsValue> {
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

    let format = gpu.get_preferred_canvas_format();
    let configuration = GpuCanvasConfiguration::new(&device, format);
    context.configure(&configuration);
    console_log!("context configured");
    
    Ok(())
}

#[wasm_bindgen(start)]
pub fn run() {
    spawn_local(async {
        run_async().await
            .unwrap_throw();
    });
}
