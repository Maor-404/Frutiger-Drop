use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct FrutigerDrop;

#[wasm_bindgen]
impl FrutigerDrop {
    #[wasm_bindgen(constructor)]
    pub fn new() -> FrutigerDrop {
        FrutigerDrop
    }
}

#[wasm_bindgen]
pub fn apply_blur(input: &[u8], width: u32, height: u32) -> Vec<u8> {
    frutiger_drop_core::apply_blur(input, width, height)
}

#[wasm_bindgen]
pub fn apply_tint(rgba: &[u8], tint: &[u8]) -> Vec<u8> {
    if tint.len() != 4 {
        wasm_bindgen::throw_str("tint must be 4 bytes (RGBA)");
    }
    frutiger_drop_core::apply_tint(rgba, (tint[0], tint[1], tint[2], tint[3]))
}

#[wasm_bindgen]
pub fn composite_layers(bottom_rgba: &[u8], top_rgba: &[u8]) -> Vec<u8> {
    frutiger_drop_core::composite_layers(bottom_rgba, top_rgba)
}

