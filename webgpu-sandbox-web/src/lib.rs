
mod console;
mod global;
mod animation;
mod scene;
mod render;
mod renderer;

use std::sync::{Arc, Mutex};

use wasm_bindgen::{prelude::*};
use wasm_bindgen_futures::{spawn_local};

use crate::render::{
    Device,
    Surface,
};
use crate::renderer::{
    Renderer,
};
use crate::scene::{
    SceneContext,
};

async fn main() -> Result<(), JsValue> {
    let device = Device::acquire().await?;
    let surface = Surface::acquire().await?;
    surface.configure(&device);
    let scene_context = SceneContext::new();
    let renderer = Renderer::new(&device, &surface, &scene_context);
    let run_loop = animation::FrameRunLoop::new(global::window(), move || {
        renderer.render_frame();
        scene_context.forward_frame(1.0 / 60.0);
    });
    run_loop.run();
    run_loop.forget();
    Ok(())
}

#[wasm_bindgen(start)]
pub fn run() {
    spawn_local(async {
        main().await
            .unwrap_throw();
    });
}
