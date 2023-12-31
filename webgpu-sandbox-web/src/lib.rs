
mod console;
mod global;
mod animation;
mod scene;
mod render;
mod renderer;
mod fetch;
mod asset;

use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::{spawn_local, JsFuture};

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
use crate::asset::Model;

async fn main() -> Result<(), JsValue> {
    console_log!("fetching model...");
    let model = Model::fetch("dragon.glb").await?
        .unwrap();
    console_log!("fetch model complete");
    let device = Device::acquire().await?;
    let surface = Surface::acquire().await?;
    surface.configure(&device);
    let scene_context = SceneContext::new(&model);
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
