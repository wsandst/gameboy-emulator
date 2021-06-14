use wasm_bindgen::prelude::*;
use emulator_core::emulator;
use hex;
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
        self.emulator.enable_bootrom();
    }

    pub fn load_save(&mut self, save_data: Vec<u8>) {
        self.emulator = emulator::Emulator::deserialize(&save_data);
    }

    pub fn save(&mut self) -> Vec<u8> {
        return self.emulator.serialize();
    }

    pub fn get_sound_queue(&mut self) -> js_sys::Float32Array {
        return js_sys::Float32Array::from(&self.emulator.get_sound_queue()[..]);
    }

    /// Returns 0 for Render event, 1 for Sound Event
    pub fn run_until_frontend_event(&mut self) -> u32 {
        match self.emulator.run_until_frontend_event() {
            emulator::FrontendEvent::Render => { return 0; }
            _ => { return 1; }
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

    pub fn get_rom_name(&mut self) -> String {
        return self.emulator.get_rom_name().to_owned();
    }

    pub fn set_rom_name(&mut self, romname: &str) {
        self.emulator.set_rom_name(romname);
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

    /// Serialize and turn the save data into a compact string representation
    pub fn save_as_str(&mut self) -> String {
        let data = self.save();
        return hex::encode(data);
    }

    /// Turn the compact string representation into save data and deserialize
    pub fn load_save_str(&mut self, string : String) {
        let save_data = hex::decode(string).unwrap();
        self.load_save(save_data);
    }
}
