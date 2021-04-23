/// Represents the gameboy screen. Generates a bitmap based on the GPU state.

use super::gpu;

use std::time::{Duration, Instant};
use std::cmp;

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
        for ly in 0..SCREEN_HEIGHT {
            self.draw_bg_line(ly, gpu.scroll_x as usize, gpu.scroll_y as usize, gpu.draw_helper.get_background_atlas());
            self.draw_bg_line(ly, gpu.window_x as usize, gpu.window_y as usize, gpu.draw_helper.get_window_atlas());
        }
    }

    pub fn draw_line(&mut self, gpu: &gpu::GPU) {
        self.draw_bg_line(gpu.ly as usize, gpu.scroll_x as usize, gpu.scroll_y as usize, gpu.draw_helper.get_background_atlas());
        if gpu.get_window_enable() {
            self.draw_bg_line(gpu.ly as usize, gpu.window_x as usize, gpu.window_y as usize, gpu.draw_helper.get_window_atlas());
        }
        if gpu.get_sprite_enable() {
            self.draw_sprite_line(gpu.ly as usize, &gpu.draw_helper.sprite_data, &gpu.draw_helper.tile_data)
        }
    }

    fn draw_bg_line(&mut self, line_y: usize, cx: usize, cy: usize, atlas : &gpu::draw_helper::TileAtlas) {
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

    fn draw_sprite_line(&mut self, line_y: usize, sprite_data: &gpu::draw_helper::SpriteData, tile_data: &gpu::draw_helper::TileData) {
        for sprite in &sprite_data.sprites {
            if self.is_sprite_within_line(line_y + 9 , &sprite) {
                let start_x = sprite.x as isize - 8;
                let tile_x = -cmp::min(start_x, 0) as usize;
                let tile_x_end = cmp::min(160 - start_x, 8) as usize;
                let tile_y = 7 - ((sprite.y) - (line_y + 9));

                for x in tile_x..tile_x_end {
                    self.bitmap[line_y*SCREEN_WIDTH*3 + (start_x as usize+x)*3+0] = tile_data.get_tile(sprite.tile_id).pixels[tile_y*8*3+x*3+0];
                    self.bitmap[line_y*SCREEN_WIDTH*3 + (start_x as usize+x)*3+1] = tile_data.get_tile(sprite.tile_id).pixels[tile_y*8*3+x*3+1];
                    self.bitmap[line_y*SCREEN_WIDTH*3 + (start_x as usize+x)*3+2] = tile_data.get_tile(sprite.tile_id).pixels[tile_y*8*3+x*3+2];
                }
            }   
        }
    }

    // Instead of subtracting 16 from y we added 16 to line_y, get underflow otherwise
    fn is_sprite_within_line(&self, line_y: usize, sprite: &gpu::draw_helper::Sprite) -> bool {
        return sprite.y > 0 && sprite.y >= line_y && sprite.y < line_y + 8 // Modify 8 to 16 to support taller sprites
    }
}

