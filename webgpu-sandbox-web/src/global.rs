
use wasm_bindgen::{prelude::*, JsCast};

use js_sys;

// @see https://github.com/gfx-rs/wgpu/blob/2209463a54321d8e8812ef3588f0cdcc65a6b3a5/wgpu/src/backend/web.rs#L1151
#[wasm_bindgen]
extern "C" {
    type Global;

    #[wasm_bindgen(method, getter, js_name = Window)]
    fn window(this: &Global) -> JsValue;

    #[wasm_bindgen(method, getter, js_name = WorkerGlobalScope)]
    fn worker(this: &Global) -> JsValue;
}

pub fn window() -> web_sys::Window {
    let global: Global = js_sys::global().unchecked_into();
    global.unchecked_into::<web_sys::Window>()
}

pub fn gpu() -> web_sys::Gpu {
    let global: Global = js_sys::global().unchecked_into();
    let gpu = if !global.window().is_undefined() {
        global.unchecked_into::<web_sys::Window>().navigator().gpu()
    } else if !global.worker().is_undefined() {
        global
            .unchecked_into::<web_sys::WorkerGlobalScope>()
            .navigator()
            .gpu()
    } else {
        panic!(
            "Accessing the GPU is only supported on the main thread or from a dedicated worker"
        );
    };
    gpu
}
