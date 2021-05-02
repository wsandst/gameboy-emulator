/// Represents the gameboy screen. Generates a bitmap based on the GPU state.

use super::gpu;
use super::gpu::draw_helper;

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
            self.draw_bg_line(ly, gpu.scroll_x as usize, gpu.scroll_y as usize, gpu, gpu.options.bg_tile_map());
            self.draw_bg_line(ly, gpu.window_x as usize, gpu.window_y as usize, gpu, gpu.options.window_tile_map());
        }
    }

    pub fn draw_line(&mut self, gpu: &gpu::GPU) {
        self.draw_bg_line(gpu.ly as usize, gpu.scroll_x as usize, gpu.scroll_y as usize, gpu, gpu.options.bg_tile_map());
        if gpu.should_draw_window() {
            self.draw_bg_line(gpu.ly as usize, gpu.window_x as usize, gpu.window_y as usize, gpu, gpu.options.window_tile_map());
        }
        if gpu.should_draw_sprites() {
            self.draw_sprite_line(gpu.ly as usize, &gpu.draw_helper, gpu.options.tile_data())
        }
    }

    fn draw_bg_line(&mut self, line_y: usize, cx: usize, cy: usize, gpu: &gpu::GPU, tilemap_select : bool) {
        let y = (line_y + cy) % 256;
        let tile_data_y = y / 8;
        let tile_y = y % 8;
        let mut color: draw_helper::Color;
        // Improvements: Remove modulo
        // Do entire tile at once
        let mut mx : u8 = cx as u8;
        for x in 0..SCREEN_WIDTH {
            let tile_id = gpu.get_tilemap_id((mx as usize) / 8, tile_data_y, tilemap_select);
            color = gpu.draw_helper.get_bg_tile_pixel(tile_id, (mx % 8) as usize, tile_y, gpu.options.tile_data());
            self.bitmap[line_y*SCREEN_WIDTH*3+x*3+0] = color.r;
            self.bitmap[line_y*SCREEN_WIDTH*3+x*3+1] = color.g;
            self.bitmap[line_y*SCREEN_WIDTH*3+x*3+2] = color.b;
            mx = mx.wrapping_add(1);
        }
    }

    fn draw_sprite_line(&mut self, line_y: usize, draw_helper: &draw_helper::DrawHelper) {
        // Clear line to white
        //self.bitmap[line_y*SCREEN_WIDTH*3..(line_y+1)*SCREEN_WIDTH*3].copy_from_slice(&[255; SCREEN_WIDTH*3]);
        for sprite in &draw_helper.sprite_data.sprites {
            if self.is_sprite_within_line(line_y + 9 , &sprite) {
                let start_x = sprite.x as isize - 8;
                let tile_x = -cmp::min(start_x, 0) as usize;
                let tile_x_end = cmp::min(cmp::max(160 - start_x,0), 8) as usize;
                let tile_y = 7 - ((sprite.
                    y) - (line_y + 9));
                let mut color : draw_helper::Color;
                for x in tile_x..tile_x_end {
                    color = draw_helper.get_sprite_tile_pixel(sprite.tile_id, x, tile_y, true, sprite.palette_select);
                    self.bitmap[line_y*SCREEN_WIDTH*3 + (start_x as usize+x)*3+0] = color.r;
                    self.bitmap[line_y*SCREEN_WIDTH*3 + (start_x as usize+x)*3+1] = color.g;
                    self.bitmap[line_y*SCREEN_WIDTH*3 + (start_x as usize+x)*3+2] = color.b;
                }
            }   
        }
    }

    // Instead of subtracting 16 from y we added 16 to line_y, get underflow otherwise
    fn is_sprite_within_line(&self, line_y: usize, sprite: &gpu::draw_helper::Sprite) -> bool {
        return sprite.y > 0 && sprite.y >= line_y && sprite.y < line_y + 8 // Modify 8 to 16 to support taller sprites
    }
}

