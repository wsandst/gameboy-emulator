/// Represents the gameboy screen. Generates a bitmap based on the GPU state.

use super::gpu;

use std::time::{Duration, Instant};

const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;

#[derive(Copy, Clone, PartialEq, Debug)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

const COLOR_WHITE: Color = Color {r:255,g:255,b:255};
const COLOR_LIGHTGRAY: Color = Color {r:170,g:170,b:170};
const COLOR_DARKGRAY: Color = Color {r:85,g:85,b:85};
const COLOR_BLACK: Color = Color {r:0,g:0,b:0};

#[derive(Copy, Clone, PartialEq, Debug)]
struct Tile {
    pixels: [u8; 8*8*3],
}

impl Tile {
    pub fn new() -> Tile {
        Tile {pixels : [255; 8*8*3] }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color : Color) {
        self.pixels[y*8*3+x*3+0] = color.r;
        self.pixels[y*8*3+x*3+1] = color.g;
        self.pixels[y*8*3+x*3+2] = color.b;
    }
}

struct Tilemap {
    pub atlas: [u8; 32*32*8*8*3],
    pub tiles: [Tile; 32*32],
}

impl Tilemap {
    pub fn new() -> Tilemap {
        Tilemap {atlas: [0; 32*32*8*8*3], tiles: [Tile::new(); 32*32]}
    }

    pub fn set_pixel(&mut self, tile_id: usize, x: usize, y: usize, color : Color) {
        self.atlas[tile_id*8*8*3 + y*8*3+x*3+0] = color.r;
        self.atlas[tile_id*8*8*3 + y*8*3+x*3+1] = color.g;
        self.atlas[tile_id*8*8*3 + y*8*3+x*3+2] = color.b;
    }

    fn generate_tilemap(&mut self, addr: usize, gpu: &gpu::GPU) {
        for y in 0..32 {
            for x in 0..32 {
                let id = gpu.read_byte(addr + y*32 + x) as usize;
                self.blit_tile_to_map(x*8, y*8, id);
                //self.blit_tile_to_map(x*8, y*8, y*32+x); // Display the tilemap
            }
        }
    }

    fn generate_for_row(&mut self, y: usize, addr: usize, data_addr: usize, gpu: &gpu::GPU, palette: &Palette) {
        for x in 0..32 {
            let id = gpu.read_byte(addr + y*32 + x) as usize;
            self.generate_tile(id, data_addr, palette, gpu);
            self.blit_tile_to_map(x*8, y*8, id);
            //self.blit_tile_to_map(x*8, y*8, y*32+x); // Display the tilemap
        }
    }

    fn generate_tile(&mut self, id: usize, addr: usize, palette : &Palette, gpu: &gpu::GPU) {
        let offset_addr = (addr - 0x8000) + id*16;
        let data : &[u8] = &gpu.video_ram[offset_addr..offset_addr+16];
        let mut y = 0;
        let mut a : u8;
        let mut b : u8;
        // This can be massively improved
        // by pre-combining the bytes using an odd/even bitmask and shifting,
        // we can half the ops.
        // Furthermore, this can be done for all colors at once, as they bit in a 64 bit integer
        // Alternatively, a lookup table can be used (8 bits is just 256 values)
        // or pext.
        for row in data.chunks(2) {
            a = row[0];
            b = row[1];
            for x in 0..8 {
                self.tiles[id].set_pixel(7-x, y, palette.palette[(a & 0x01 | ((b & 0x01) << 1)) as usize]);
                a = a >> 1;
                b = b >> 1;
            }
            y += 1;
        }
    }

    fn blit_tile_to_map(&mut self, x: usize, y: usize, tile_id: usize) {
        let mut i: usize;
        for iy in 0..8 {
            i = (y+iy) * 256 * 3 + x*3;
            // Memcpy the row
            self.atlas[i..i+8*3].copy_from_slice(&self.tiles[tile_id].pixels[iy*8*3..iy*8*3+8*3]);
        }
    }
}

struct Palette {
    palette: [Color; 4]
}

impl Palette {
    fn new() -> Palette {
        return Palette { palette: [COLOR_WHITE, COLOR_LIGHTGRAY, COLOR_DARKGRAY, COLOR_BLACK]}
    }

    pub fn update(&mut self, palette_flag: u8) {
        self.palette[0] = self.get_color_from_bits(palette_flag & 0b0000_0011);
        self.palette[1] = self.get_color_from_bits((palette_flag & 0b0000_1100) >> 2);
        self.palette[2] = self.get_color_from_bits((palette_flag & 0b0011_0000) >> 4);
        self.palette[3] = self.get_color_from_bits((palette_flag & 0b1100_0000) >> 6);
    }

    fn get_color_from_bits(&mut self, color_val: u8) -> Color {
        match color_val {
            0 => COLOR_WHITE,
            1 => COLOR_LIGHTGRAY,
            2 => COLOR_DARKGRAY,
            3 => COLOR_BLACK,
            _ => panic!("Invalid color.")
        }
    }
}


pub struct Screen {
    pub bitmap: [u8; SCREEN_HEIGHT*SCREEN_WIDTH*3], // 160*144 screen, 4 channels
    tilemap1: Tilemap,
    tilemap2: Tilemap,
    background_palette: Palette,
    sprite_palette_1: Palette,
    sprite_palette_2: Palette,
    using_tilemap_1: bool,
    using_tiledata_1: bool,
    pub recache_tiles_per_row: bool,
}

impl Screen {
    pub fn new() -> Screen {
        Screen { bitmap: [0; SCREEN_HEIGHT*SCREEN_WIDTH*3], 
            tilemap1 : Tilemap::new(),
            tilemap2 : Tilemap::new(),
            background_palette : Palette::new(),
            sprite_palette_1 : Palette::new(),
            sprite_palette_2 : Palette::new(),
            using_tiledata_1: true,
            using_tilemap_1: true,
            recache_tiles_per_row: false}
    }

    pub fn draw_frame(&mut self, gpu: &gpu::GPU) {
        self.update_palettes(gpu);
        self.cache_tiles(gpu); // Perf: Takes about half the time
        let cy = gpu.scroll_y as usize;
        let cx = gpu.scroll_x as usize;
        for ly in 0..SCREEN_HEIGHT {
            self.blit_line(ly, cx, cy , gpu);
        }
    }

    pub fn draw_line(&mut self, gpu: &gpu::GPU) {
        self.update_palettes(gpu);
        self.cache_tiles_by_line(gpu.ly as usize, gpu.scroll_x as usize, gpu.scroll_y as usize, gpu);
        self.blit_line(gpu.ly as usize, gpu.scroll_x as usize, gpu.scroll_y as usize, gpu);
    }

    fn blit_line(&mut self, line_y: usize, cx: usize, cy: usize, gpu: &gpu::GPU) {
        // Identify the relevant tile row, starting x
        // Then go through every tile in order
        // Memcpy the line. Start and end will overshoot. Special logic for those memcpy
        let i = line_y * SCREEN_WIDTH * 3;
        let it = ((line_y+cy)%255) * 256 * 3;
        self.bitmap[i..i+SCREEN_WIDTH*3].copy_from_slice(&self.tilemap1.atlas[it..it+SCREEN_WIDTH*3]);
    }

    fn cache_tiles_by_line(&mut self, line_y: usize, cx: usize, cy: usize, gpu: &gpu::GPU) {
        let ty = ((line_y + cy) % 255) / 8;
        if gpu.state_modified || self.recache_tiles_per_row && (line_y+cy % 255) % 8 == 0 {
            self.recache_tiles_per_row = true;
            self.using_tiledata_1 = gpu.get_background_tile_data_select();
            self.using_tilemap_1 = !gpu.get_background_tile_map_select();
            let ty = ((line_y + cy) % 255) / 8;
            if self.using_tiledata_1 {
                if self.using_tilemap_1 {
                    self.tilemap1.generate_for_row(ty, 0x9800, 0x8000, gpu, &self.background_palette);
                }
                else {
                    self.tilemap1.generate_for_row(ty, 0x9C00, 0x8000, gpu, &self.background_palette);
                }
            }
            else {
                if self.using_tilemap_1 {
                    self.tilemap2.generate_for_row(ty, 0x9800, 0x8800, gpu, &self.background_palette);
                }
                else {
                    self.tilemap2.generate_for_row(ty, 0x9C00, 0x8000, gpu, &self.background_palette);
                }
            }
        }
    }

    pub fn cache_tiles(&mut self, gpu: &gpu::GPU) {
        if gpu.state_modified {
            self.using_tiledata_1 = gpu.get_background_tile_data_select();
            self.using_tilemap_1 = !gpu.get_background_tile_map_select();
            if self.using_tiledata_1 {
                for id in 0..256 {
                    self.tilemap1.generate_tile(id, 0x8000, &self.background_palette, gpu);
                }
                if self.using_tilemap_1 {
                    self.tilemap1.generate_tilemap(0x9800, gpu);
                }
                else {
                    self.tilemap1.generate_tilemap(0x9C00, gpu);
                }
            }
            else {
                for id in 0..256 {
                    self.tilemap2.generate_tile(id, 0x8800, &self.background_palette, gpu);
                }
                if self.using_tilemap_1 {
                    self.tilemap2.generate_tilemap(0x9800, gpu);
                }
                else {
                    self.tilemap2.generate_tilemap(0x9C00, gpu);
                }
            }

        }
    }

    // Palette updating
    fn update_palettes(&mut self, gpu: &gpu::GPU) {
        self.background_palette.update(gpu.background_palette);
        self.sprite_palette_1.update(gpu.sprite_palette_1);
        self.sprite_palette_2.update(gpu.sprite_palette_2);
    } 
}

