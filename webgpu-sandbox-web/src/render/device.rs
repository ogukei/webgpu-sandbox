
use std::sync::Arc;

use crate::{
    global,
};

// @see https://rustwasm.github.io/wasm-bindgen/examples/wasm-in-wasm.html
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::{JsFuture};

use web_sys::{
    Gpu,
    GpuAdapter,
    GpuDevice,
};

pub struct Device {
    gpu: Gpu,
    device: GpuDevice,
}

impl Device {
    pub async fn acquire() -> Result<Arc<Self>, JsValue> {
        let gpu = global::gpu();
        let adapter = JsFuture::from(gpu.request_adapter()).await?;
        let adapter: GpuAdapter = adapter.unchecked_into();
        let device = JsFuture::from(adapter.request_device()).await?;
        let device: GpuDevice = device.unchecked_into();
        Ok(Self::new(gpu, device))
    }

    fn new(gpu: Gpu, device: GpuDevice) -> Arc<Self> {
        let this = Self {
            gpu,
            device,
        };
        Arc::new(this)
    }

    pub fn gpu(&self) -> &Gpu {
        &self.gpu
    }

    pub fn device(&self) -> &GpuDevice {
        &self.device
    }
}
