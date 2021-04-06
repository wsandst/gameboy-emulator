use wasm_bindgen::prelude::*;
use emulator_core::emulator;
// Javascript interface to emulator core

#[wasm_bindgen]
extern {
    pub fn alert(s: &str);
}

#[wasm_bindgen]
pub fn test() {
    let mut emulator = emulator::Emulator::new();
    alert(&format!("Hello, {}!", emulator.js_test()))
}