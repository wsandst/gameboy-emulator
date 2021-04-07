/// Represents the gameboy screen. Generates a bitmap based on the GPU state.

use super::gpu;

const SCREEN_WIDTH: usize = 10;
const SCREEN_HEIGHT: usize = 10;


struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

pub struct Screen {
    bitmap: [u8; SCREEN_HEIGHT*SCREEN_WIDTH*4], // 160*144 screen, 4 channels
}

impl Screen {
    pub fn new() -> Screen {
        Screen { bitmap: [0; SCREEN_HEIGHT*SCREEN_WIDTH*4]}
    }

    pub fn render_frame(&mut self, gpu: &gpu::GPU) {

    }

    fn set_pixel(&mut self, x : usize, y: usize, color : Color) {
        let i = y * SCREEN_HEIGHT * 4 + x * 4;
        self.bitmap[i] = color.r;
        self.bitmap[i+1] = color.g;
        self.bitmap[i+2] = color.b;
        self.bitmap[i+3] = color.a;
    }
}

