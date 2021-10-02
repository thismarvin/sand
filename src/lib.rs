use wasm_bindgen::prelude::*;

fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn add(a: f32, b: f32) -> f32 {
    set_panic_hook();
    a + b
}
