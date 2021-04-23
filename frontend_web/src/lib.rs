use wasm_bindgen::prelude::*;
use emulator_core::emulator;
// Javascript interface to emulator core

const SCREEN_WIDTH : usize = 160;
const SCREEN_HEIGHT : usize = 144;

#[wasm_bindgen]
extern {
    pub fn alert(s: &str);
}

#[wasm_bindgen]
pub struct EmulatorWrapper {
    emulator : emulator::Emulator,
}

#[wasm_bindgen]
impl EmulatorWrapper {
    pub fn new() -> EmulatorWrapper {
        EmulatorWrapper { emulator: emulator::Emulator::new()}
    }

    pub fn load_rom(&mut self, rom_data : Vec<u8>) {
        self.emulator.load_rom_from_vec(&rom_data);
    }

    pub fn run_until_draw(&mut self) {
        self.emulator.run_until_draw();
    }

    pub fn get_screen_bitmap(&mut self) -> Vec<u8>  {
        let mut bitmap : Vec<u8> = vec![255; SCREEN_WIDTH*SCREEN_HEIGHT*4];
        for i in 0..SCREEN_WIDTH*SCREEN_HEIGHT {
            bitmap[i*4 + 0] = self.emulator.screen.bitmap[i*3 + 0];
            bitmap[i*4 + 1] = self.emulator.screen.bitmap[i*3 + 1];
            bitmap[i*4 + 2] = self.emulator.screen.bitmap[i*3 + 2];
            bitmap[i*4 + 3] = 255;
        }
        return bitmap;
    }
}