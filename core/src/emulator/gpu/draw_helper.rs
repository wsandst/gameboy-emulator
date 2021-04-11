use super::super::gpu;

const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
}

const COLOR_WHITE: Color = Color {r:255,g:255,b:255};
const COLOR_LIGHTGRAY: Color = Color {r:170,g:170,b:170};
const COLOR_DARKGRAY: Color = Color {r:85,g:85,b:85};
const COLOR_BLACK: Color = Color {r:0,g:0,b:0};

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Tile {
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

    fn generate(&mut self, id: usize, addr: usize, palette : &Palette, gpu_vram : &[u8; 8192]) {
        let offset_addr = (addr - 0x8000) + id*16;
        let data : &[u8] = &gpu_vram[offset_addr..offset_addr+16];
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
                self.set_pixel(7-x, y, palette.palette[(a & 0x01 | ((b & 0x01) << 1)) as usize]);
                a = a >> 1;
                b = b >> 1;
            }
            y += 1;
        }
    }
}

pub struct TileAtlas {
    pub atlas: Vec<u8>,
}

impl TileAtlas {
    pub fn new() -> TileAtlas {
        TileAtlas { atlas : vec![0; 32*32*8*8*3] }
    }

    pub fn generate(&mut self, addr: usize, tiles : &[Tile; 32*32], gpu_vram : &[u8; 8192]) {
        for y in 0..32 {
            for x in 0..32 {
                let id = gpu_vram[addr + y*32 + x - 0x8000] as usize;
                self.blit_tile_to_map(x*8, y*8, id, tiles);
                //self.blit_tile_to_map(x*8, y*8, y*32+x); // Display the tilemap
            }
        }
    }

    pub fn blit_tile_to_map(&mut self, x: usize, y: usize, tile_id: usize, tiles : &[Tile; 32*32]) {
        let mut i: usize;
        for iy in 0..8 {
            i = (y+iy) * 256 * 3 + x*3;
            // Memcpy the row
            self.atlas[i..i+8*3].copy_from_slice(&tiles[tile_id].pixels[iy*8*3..iy*8*3+8*3]);
        }
    }
}

pub struct DrawHelper {
    pub atlas1 : TileAtlas,
    pub atlas2 : TileAtlas,
    pub tiles: [Tile; 32*32],
    pub background_palette : Palette,
    pub sprite_palette_1: Palette,
    pub sprite_palette_2: Palette,
}

impl DrawHelper {
    pub fn new() -> DrawHelper {
        println!("Making new DrawHelper");
        DrawHelper { atlas1: TileAtlas::new(), atlas2: TileAtlas::new(), 
            tiles: [Tile::new(); 32*32], background_palette : Palette::new(), sprite_palette_1 : Palette::new(),
            sprite_palette_2 : Palette::new() }
    }

    pub fn generate_atlas(&mut self, gpu_vram: &[u8; 8192]) {
        self.atlas1.generate(0x9800, &self.tiles, gpu_vram);
    }

    pub fn generate_tiles(&mut self, gpu_vram: &[u8; 8192]) {
        for id in 0..256 {
            self.tiles[id].generate(id, 0x8000, &self.background_palette, gpu_vram);
        }
    }
}

pub struct Palette {
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