

use crate::global;

use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::{JsFuture};

use web_sys::{
    RequestInit,
    Request,
    RequestMode,
    Response,
};

use js_sys::{ArrayBuffer, Uint8Array};

pub async fn fetch(url: &str) -> Result<Vec<u8>, JsValue> {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);
    let request = Request::new_with_str_and_init(&url, &opts)?;
    request
        .headers()
        .set("Accept", "application/octet-stream")?;
    let window = global::window();
    let response = JsFuture::from(window.fetch_with_request(&request)).await?;
    let response: Response = response.unchecked_into();
    let array_buffer = JsFuture::from(response.array_buffer()?).await?;
    let array_buffer: ArrayBuffer = array_buffer.unchecked_into();
    let array_buffer = Uint8Array::new(&array_buffer);
    let bytes = array_buffer.to_vec();
    Ok(bytes)
}
