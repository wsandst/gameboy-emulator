/// Represents the gameboy screen. Generates a bitmap based on the GPU state.

use super::gpu;

use std::time::{Duration, Instant};

const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;


pub struct Screen {
    pub bitmap: [u8; SCREEN_HEIGHT*SCREEN_WIDTH*3], // 160*144 screen, 4 channels
}

impl Screen {
    pub fn new() -> Screen {
        Screen { bitmap: [255; SCREEN_HEIGHT*SCREEN_WIDTH*3] }
    }

    pub fn draw_frame(&mut self, gpu: &gpu::GPU) {
        let cy = gpu.scroll_y as usize;
        let cx = gpu.scroll_x as usize;
        for ly in 0..SCREEN_HEIGHT {
            self.blit_line(ly, cx, cy, gpu.draw_helper.get_background_atlas());
        }
    }

    pub fn draw_line(&mut self, gpu: &gpu::GPU) {
        self.blit_line(gpu.ly as usize, gpu.scroll_x as usize, gpu.scroll_y as usize, gpu.draw_helper.get_background_atlas());
        self.blit_line(gpu.ly as usize, gpu.window_x as usize, gpu.window_y as usize, gpu.draw_helper.get_window_atlas());
    }

    fn blit_line(&mut self, line_y: usize, cx: usize, cy: usize, atlas : &gpu::draw_helper::TileAtlas) {
        // Identify the relevant tile row, starting x
        // Then go through every tile in order
        // Memcpy the line. Start and end will overshoot. Special logic for those memcpy
        let i = line_y * SCREEN_WIDTH * 3;
        let it = ((line_y+cy)%255) * 256 * 3 + cx*3;
        if cx <= (255-SCREEN_WIDTH) { // Line completely overlaps the atlas, only one memcpy needed
            self.bitmap[i..i+SCREEN_WIDTH*3].copy_from_slice(&atlas.atlas[it..it+SCREEN_WIDTH*3]);
        }
        else { // The line wraps around, have to use two memcopys
            let width_right = 255-cx;
            let new_it = ((line_y+cy)%255) * 256 * 3;
            // Right section
            self.bitmap[i..i+width_right*3].copy_from_slice(&atlas.atlas[it..it+SCREEN_WIDTH*3]);
            // Left section
            self.bitmap[i+width_right*3..i+width_right*3+(SCREEN_WIDTH-width_right)*3].copy_from_slice(
                &atlas.atlas[it..it+SCREEN_WIDTH*3]);
        }

    }
}

