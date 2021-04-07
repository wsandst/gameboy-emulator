/// Represents the gameboy screen. Generates a bitmap based on the GPU state.

use super::gpu;

const SCREEN_WIDTH: usize = 256;
const SCREEN_HEIGHT: usize = 256;

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
    pixels: [Color; 8*8],
}

impl Tile {
    pub fn get_pixel(&self, x: usize, y: usize) -> Color {
        return self.pixels[y*8+x];
    }
}

pub struct Screen {
    pub bitmap: [u8; SCREEN_HEIGHT*SCREEN_WIDTH*3], // 160*144 screen, 4 channels
    tiles: [Tile; 32*32],
    background_palette: [Color; 4],
}

impl Screen {
    pub fn new() -> Screen {
        Screen { bitmap: [0; SCREEN_HEIGHT*SCREEN_WIDTH*3], 
            tiles : [Tile {pixels : [COLOR_WHITE; 8*8]} ;32*32],
            background_palette : [COLOR_WHITE, COLOR_LIGHTGRAY, COLOR_DARKGRAY, COLOR_BLACK]}
    }

    pub fn render_frame(&mut self, gpu: &gpu::GPU) {
        self.cache_tiles(gpu);
        for y in 0..32 {
            for x in 0..32 {
                let id = gpu.read_byte(0x9800 + y*32 + x) as usize;
                self.blit_background_tile(x*8, y*8, id);
            }
        }
    }

    fn set_pixel(&mut self, x : usize, y: usize, color : Color) {
        let i = y * SCREEN_HEIGHT * 3 + x * 3;
        self.bitmap[i] = color.r;
        self.bitmap[i+1] = color.g;
        self.bitmap[i+2] = color.b;
    }

    fn cache_tiles(&mut self, gpu: &gpu::GPU) {
        for id in 0..128 {
            self.create_tile(gpu, id);
        }
    }

    fn init_palette(&mut self, gpu: &gpu::GPU) {

    }   

    fn create_tile(&mut self, gpu: &gpu::GPU, id: usize) {
        let addr : usize = (id as usize)*16;
        let data : &[u8] = &gpu.video_ram[addr..addr+16];
        let mut y = 0;
        let mut a : u8;
        let mut b : u8;
        for row in data.chunks(2) {
            a = row[0];
            b = row[1];
            for x in 0..8 {
                self.tiles[id].pixels[y*8+x] = self.background_palette[(a & 0x01 | ((a & 0x01) << 1)) as usize];
                a = a >> 1;
                b = b >> 1;
            }
            y += 1;
        }
    }


    fn blit_background_tile(&mut self, x: usize, y: usize, tile_id: usize) {
        for iy in 0..8 {
            for ix in 0..8 {
                self.set_pixel(x+ix, y+iy, self.tiles[tile_id].get_pixel(ix, iy));
            }
        }
    }
}

