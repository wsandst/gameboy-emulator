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
        EmulatorWrapper { emulator: emulator::Emulator::new(false)}
    }

    pub fn load_rom(&mut self, rom_data : Vec<u8>) {
        self.emulator.load_rom_from_data(&rom_data);
    }

    pub fn load_bootrom(&mut self, bootrom_data: Vec<u8>) {
        self.emulator.load_bootrom_from_data(&bootrom_data);
    }

    pub fn load_save(&mut self, save_data: Vec<u8>) {
        self.emulator = emulator::Emulator::deserialize(&save_data);
    }

    pub fn run_until_frontend_event(&mut self) {
        loop {
            match self.emulator.run_until_frontend_event() {
                emulator::FrontendEvent::Render => { break; }
                _ => { }
            }
        }
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

    // Key input
    // Key down
    pub fn press_key_up(&mut self) {
        self.emulator.press_key(emulator::KeyPress::Up);
    }

    pub fn press_key_down(&mut self) {
        self.emulator.press_key(emulator::KeyPress::Down);
    }

    pub fn press_key_left(&mut self) {
        self.emulator.press_key(emulator::KeyPress::Left);
    }

    pub fn press_key_right(&mut self) {
        self.emulator.press_key(emulator::KeyPress::Right);
    }

    pub fn press_key_a(&mut self) {
        self.emulator.press_key(emulator::KeyPress::A);
    }

    pub fn press_key_b(&mut self) {
        self.emulator.press_key(emulator::KeyPress::B);
    }

    pub fn press_key_start(&mut self) {
        self.emulator.press_key(emulator::KeyPress::Start);
    }

    pub fn press_key_select(&mut self) {
        self.emulator.press_key(emulator::KeyPress::Select);
    }

    // Key up
    pub fn clear_key_up(&mut self) {
        self.emulator.clear_key(emulator::KeyPress::Up);
    }

    pub fn clear_key_down(&mut self) {
        self.emulator.clear_key(emulator::KeyPress::Down);
    }

    pub fn clear_key_left(&mut self) {
        self.emulator.clear_key(emulator::KeyPress::Left);
    }

    pub fn clear_key_right(&mut self) {
        self.emulator.clear_key(emulator::KeyPress::Right);
    }

    pub fn clear_key_a(&mut self) {
        self.emulator.clear_key(emulator::KeyPress::A);
    }

    pub fn clear_key_b(&mut self) {
        self.emulator.clear_key(emulator::KeyPress::B);
    }

    pub fn clear_key_start(&mut self) {
        self.emulator.clear_key(emulator::KeyPress::Start);
    }

    pub fn clear_key_select(&mut self) {
        self.emulator.clear_key(emulator::KeyPress::Select);
    }
}