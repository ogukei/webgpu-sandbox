
mod console;
mod gpu;

use gpu::gpu;

// @see https://rustwasm.github.io/wasm-bindgen/examples/wasm-in-wasm.html
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::{spawn_local, JsFuture};

use web_sys::{GpuAdapter, GpuDevice};

async fn main() -> Result<(), JsValue> {
    console_log!("initializing...");
    let gpu = gpu();
    let adapter = JsFuture::from(gpu.request_adapter()).await?;
    let adapter: GpuAdapter = adapter.unchecked_into();
    let device = JsFuture::from(adapter.request_device()).await?;
    let device: GpuDevice = device.unchecked_into();
    console_log!("device acquired.");
    Ok(())
}

#[wasm_bindgen(start)]
pub fn entry() {
    spawn_local(async {
        main().await
            .unwrap_throw();
    });
}
