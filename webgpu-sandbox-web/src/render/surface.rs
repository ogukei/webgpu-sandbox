

use std::sync::{Arc, Mutex};

use crate::{
    global,
};

use crate::render::device::Device;

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

pub struct Surface {
    window: web_sys::Window,
    canvas: web_sys::HtmlCanvasElement,
    canvas_context: GpuCanvasContext,
    configuration: Mutex<Option<SurfaceConfiguration>>,
}

impl Surface {
    pub async fn acquire() -> Result<Arc<Self>, JsValue> {
        let window = global::window();
        let document = window.document().unwrap();
        let canvas = document.get_element_by_id("canvas").unwrap();
        let canvas: web_sys::HtmlCanvasElement = canvas.unchecked_into();
        let context = canvas.get_context("webgpu").unwrap().unwrap();
        let context: GpuCanvasContext = context.unchecked_into();
        Ok(Self::new(window, canvas, context))
    }

    fn new(window: web_sys::Window, 
           canvas: web_sys::HtmlCanvasElement,
           canvas_context: GpuCanvasContext) -> Arc<Self> {
        let this = Self {
            window,
            canvas,
            canvas_context,
            configuration: Mutex::new(None),
        };
        Arc::new(this)
    }

    pub fn canvas_context(&self) -> &GpuCanvasContext {
        &self.canvas_context
    }

    pub fn configure(&self, device: &Device) {
        let window = &self.window;
        let canvas = &self.canvas;
        let device_pixel_ratio = window.device_pixel_ratio();
        let width = (canvas.client_width() as f64 * device_pixel_ratio) as usize;
        let height = (canvas.client_height() as f64 * device_pixel_ratio) as usize;
        // configure canvas size
        canvas.set_width(width as u32);
        canvas.set_height(height as u32);
        // format
        let gpu = device.gpu();
        let presentation_format = gpu.get_preferred_canvas_format();
        let mut canvas_configuration = GpuCanvasConfiguration::new(device.device(), presentation_format);
        canvas_configuration.alpha_mode(GpuCanvasAlphaMode::Opaque);
        self.canvas_context.configure(&canvas_configuration);
        // store current configuration
        let mut configuration = self.configuration.lock().unwrap();
        *configuration = Some(SurfaceConfiguration {
            width,
            height,
            presentation_format,
        });
    }

    pub fn configuration(&self) -> SurfaceConfiguration {
        let configuration = self.configuration.lock().unwrap();
        configuration.unwrap()
    }
}

#[derive(Copy, Clone)]
pub struct SurfaceConfiguration {
    width: usize,
    height: usize,
    presentation_format: GpuTextureFormat,
}

impl SurfaceConfiguration {
    pub fn presentation_format(&self) -> GpuTextureFormat {
        self.presentation_format
    }

    pub fn presentation_size(&self) -> js_sys::Array {
        let presentation_size: Vec<JsValue> = vec![self.width as f64, self.height as f64].into_iter().map(Into::into).collect();
        let presentation_size = presentation_size.into_iter().collect::<js_sys::Array>();
        presentation_size
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }
}
