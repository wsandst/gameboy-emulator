/// Represents the gameboy screen. Generates a bitmap based on the GPU state.

use super::gpu;
use super::gpu::draw_helper;

use std::cmp;

const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;

use serde::{Serialize, Deserialize};
use serde_big_array::BigArray;

#[derive(Serialize, Deserialize)]
pub struct Screen {
    #[serde(with = "BigArray")]
    pub bitmap: [u8; SCREEN_HEIGHT*SCREEN_WIDTH*3], // 160*144 screen, 4 channels
}

impl Screen {
    pub fn new() -> Screen {
        Screen { 
            bitmap: [255; SCREEN_HEIGHT*SCREEN_WIDTH*3], 
        }
    }

    pub fn draw_frame(&mut self, gpu: &gpu::GPU) {
        for ly in 0..SCREEN_HEIGHT {
            self.draw_bg_line(ly, gpu.scroll_x as usize, gpu.scroll_y as usize, gpu, gpu.get_bg_tile_map());
            self.draw_bg_line(ly, gpu.window_x as usize, gpu.window_y as usize, gpu, gpu.get_window_tile_map());
        }
    }

    /// Draw a scanline to the bitmap, 
    /// consisting of a background layer, a sprite layer and a window layer
    pub fn draw_line(&mut self, gpu: &gpu::GPU) {
        if gpu.get_bg_enable() {
            // Draw the background layer
            self.draw_bg_line(gpu.ly as usize, gpu.scroll_x as usize, gpu.scroll_y as usize, gpu, gpu.get_bg_tile_map());
            if gpu.should_draw_window() {
                // Draw the window layer
                self.draw_window_line(gpu.ly as usize, gpu.window_x as usize, gpu.window_y as usize, gpu, gpu.get_window_tile_map());
            }
        }
        if gpu.should_draw_sprites() {
            // Draw 8x8 sprites
            if !gpu.get_sprite_tile_size() {
                // Draw 8x8 sprites
                self.draw_sprite_line(gpu.ly as usize, &gpu.draw_helper)
            }
            else { 
                // Draw 8x16 sprites
                self.draw_double_sprite_line(gpu.ly as usize, &gpu.draw_helper)
            }
        }
    }

    /// Draw a line of the background layer, which is a slice of a 256x256 tilemap
    /// which wraps around
    /// cx and cy is the background scroll position
    /// tilemap_select selects which tilemap to use
    fn draw_bg_line(&mut self, line_y: usize, cx: usize, cy: usize, gpu: &gpu::GPU, tilemap_select : bool) {
        let y = (line_y + cy) % 256;
        let tile_data_y = y / 8;
        let tile_y = y % 8;
        let mut color: draw_helper::Color;
        // Improvements: Remove modulo
        // Do entire tile at once
        //println!{"{}", cx}
        let mut mx : u8 = cx as u8;
        // Go through every x position in the line, determine which tile to use from the tilemap
        // and pick a pixel from it
        for x in 0..SCREEN_WIDTH {
            let tile_id = gpu.get_tilemap_id((mx as usize) / 8, tile_data_y, tilemap_select);
            color = gpu.draw_helper.get_bg_tile_pixel(tile_id, (mx % 8) as usize, tile_y, gpu.get_tile_data());
            self.bitmap[line_y*SCREEN_WIDTH*3+x*3+0] = color.r;
            self.bitmap[line_y*SCREEN_WIDTH*3+x*3+1] = color.g;
            self.bitmap[line_y*SCREEN_WIDTH*3+x*3+2] = color.b;
            mx = mx.wrapping_add(1);
        }
    }

    /// Draw a line of the window layer. This window starts to be drawn at cx, cy
    /// and covers everything underneath.
    fn draw_window_line(&mut self, line_y: usize, cx: usize, cy: usize, gpu: &gpu::GPU, tilemap_select : bool) {
        if line_y < cy { 
            // No need to draw this line if window starts further down
            return;
        }
        let y = line_y - cy;
        let tile_data_y = y / 8;
        let tile_y = y % 8;
        let mut color: draw_helper::Color;
        // If cx is less than 7, we need to start from a later x in the window
        // as part of the window is outside the view
        let mut mx = cmp::max(-(cx as isize - 7), 0) as usize;
        // Which x should we start drawing on the bitmap, while not underflowing when
        // window is outside the bitmap?
        for x in (cmp::max(cx as isize - 7, 0) as usize)..SCREEN_WIDTH {
            let tile_id = gpu.get_tilemap_id(mx / 8, tile_data_y, tilemap_select);
            color = gpu.draw_helper.get_bg_tile_pixel(tile_id, mx % 8, tile_y, gpu.get_tile_data());
            self.bitmap[line_y*SCREEN_WIDTH*3+x*3+0] = color.r;
            self.bitmap[line_y*SCREEN_WIDTH*3+x*3+1] = color.g;
            self.bitmap[line_y*SCREEN_WIDTH*3+x*3+2] = color.b;
            mx += 1;
        }
    }

    /// Draw a line of 8x8 sprites
    /// Every line can have a max of 10 sprites
    /// On DMG they should generally be picked based on x-sorting, but this seemed
    /// annoying to implement so priority only uses sprite id order currently
    fn draw_sprite_line(&mut self, line_y: usize, draw_helper: &draw_helper::DrawHelper) {
        let mut sprite_count = 0;
        // Go through all 40 sprites
        for sprite in &draw_helper.sprite_data.sprites {
            if sprite_count >= 10 { // Only 10 sprites can be drawn per line
                return;
            }
            // Is this sprite within the scanline?
            if self.is_sprite_within_line(line_y + 9 , &sprite, 8) {
                // If so, draw the tile row to the bitmap
                let start_x = sprite.x as isize - 8;
                // Ugly code to prevent out of bounds accesses to bitmap
                let tile_x = -cmp::min(start_x, 0) as usize;
                let tile_x_end = cmp::min(cmp::max(160 - start_x, 0), 8) as usize;
                let tile_y = 7 - ((sprite.
                    y) - (line_y + 9));
                let mut color : draw_helper::Color;
                // Go through every pixel in the tile and add it to the bitmap
                for x in tile_x..tile_x_end {
                    color = draw_helper.get_sprite_tile_pixel(sprite.tile_id, x, tile_y, true, sprite, false);
                    let bitmap_index = line_y*SCREEN_WIDTH*3 + ((start_x + x as isize) as usize)*3;
                    if color.a > 0 && (!sprite.below_background || self.bitmap[bitmap_index+0] == draw_helper.background_palette.get_color(0).r) { // Skip transparent pixels
                        self.bitmap[bitmap_index+0] = color.r;
                        self.bitmap[bitmap_index+1] = color.g;
                        self.bitmap[bitmap_index+2] = color.b;
                    }
                }
                sprite_count += 1;
            }   
        }
    }

    /// Draw a line of 8x16 sprites
    /// This is similar to the 8x8 method, except we need to keep track of two
    /// tiles and also need to handle flip_y a bit differently.
    /// The second tile used is the tile directly to the right in memory
    fn draw_double_sprite_line(&mut self, line_y: usize, draw_helper: &draw_helper::DrawHelper) {
        // Clear line to white
        //self.bitmap[line_y*SCREEN_WIDTH*3..(line_y+1)*SCREEN_WIDTH*3].copy_from_slice(&[255; SCREEN_WIDTH*3]);
        let mut sprite_count = 0;
        for sprite in &draw_helper.sprite_data.sprites {
            if self.is_sprite_within_line(line_y+1, &sprite, 16) {
                if sprite_count >= 10 { // Only 10 sprites can be drawn per line
                    return;
                }
                let start_x = sprite.x as isize - 8;
                let tile_x = -cmp::min(start_x, 0) as usize;
                let tile_x_end = cmp::min(cmp::max(160 - start_x, 0), 8) as usize;
                let mut tile_y: usize;
                if !sprite.flip_y {
                    tile_y = 15 - (sprite.
                        y - (line_y+1));
                }
                else {
                    tile_y = sprite.y - (line_y+1);
                }
                let tile_id : u8;
                if tile_y > 7 {
                    tile_id = (sprite.tile_id & 0b1111_1110) + 1;
                }
                else {
                    tile_id = sprite.tile_id & 0b1111_1110;
                }
                tile_y = tile_y % 8;
                let mut color : draw_helper::Color;
                for x in tile_x..tile_x_end {
                    color = draw_helper.get_sprite_tile_pixel(tile_id, x, tile_y, true, sprite, true);
                    let bitmap_index = line_y*SCREEN_WIDTH*3 + ((start_x + x as isize) as usize)*3;
                    if color.a > 0 { // Skip transparent pixels
                        self.bitmap[bitmap_index+0] = color.r;
                        self.bitmap[bitmap_index+1] = color.g;
                        self.bitmap[bitmap_index+2] = color.b;
                    }
                }
                sprite_count += 1;
            }   
        }
    }


    // Instead of subtracting 16 from y we added 16 to line_y, get underflow otherwise
    fn is_sprite_within_line(&self, line_y: usize, sprite: &gpu::draw_helper::Sprite, height: usize) -> bool {
        return sprite.y > 0 && sprite.y >= line_y && sprite.y < line_y + height
    }

}

