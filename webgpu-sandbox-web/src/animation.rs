
use std::sync::{Arc, Mutex};

use wasm_bindgen::{prelude::*, JsCast};
use web_sys::Window;

pub struct FrameRunLoop {
    window: Window,
    closure: Mutex<Option<Closure<dyn FnMut()>>>,
    cyclic_reference: Mutex<Option<Arc<Self>>>,
}

impl FrameRunLoop {
    pub fn new<F: Fn() + 'static>(window: Window, f: F) -> Arc<Self> {
        let this = Self {
            window,
            closure: Mutex::new(None),
            cyclic_reference: Mutex::new(None),
        };
        let this = Arc::new(this);
        // assign closure
        let this_weak = Arc::downgrade(&this);
        let closure = Box::new(move || {
            f();
            let Some(this) = this_weak.upgrade() else { return };
            this.on_frame();
        });
        let closure = Closure::wrap(closure as Box<dyn FnMut()>);
        if let Ok(mut mutex) = this.closure.lock() {
            *mutex = Some(closure);
        }
        this
    }

    fn on_frame(&self) {
        self.request_animation_frame();
    }

    fn request_animation_frame(&self) {
        let Ok(closure) = self.closure.lock() else { return };
        let Some(closure) = closure.as_ref() else { return };
        _ = self.window.request_animation_frame(closure.as_ref().unchecked_ref());
    }

    pub fn run(&self) {
        self.request_animation_frame();
    }

    // intentionally make the reference leak
    pub fn forget(self: &Arc<Self>) {
        let Ok(mut reference) = self.cyclic_reference.lock() else { return };
        *reference = Some(Arc::clone(self));
    }
}

