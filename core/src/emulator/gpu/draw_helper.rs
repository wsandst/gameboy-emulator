/// Contains acceleration structures for Drawing, based on
/// the tile memory in the GPU/PPU VRAM. These cache the gameboy represenations
/// in a better format for modern rendering.
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

/// Represents a 8x8 tile of Color
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

    /// Generate the tile from the Tile representation in GPU VRAM
    /// The lower and upper bits for the color are in separate bytes,
    /// which makes the parsing somewhat convoluted
    fn generate(&mut self, id: usize, addr: usize, palette : &Palette, gpu_vram : &[u8; 8192]) {
        let offset_addr = (addr - 0x8000) + id*16;
        let data : &[u8] = &gpu_vram[offset_addr..offset_addr+16];
        let mut y = 0;
        let mut a : u8;
        let mut b : u8;
        // This can be massively improved by pre-combining the bytes 
        // using an odd/even bitmask and shifting, which halfs the ops.
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

/// Represents the 32x32 Tile Atlas, based on the Tilemap and Tiledata in the GPU/PPU
pub struct TileAtlas {
    pub atlas: Vec<u8>,
}

impl TileAtlas {
    pub fn new() -> TileAtlas {
        TileAtlas { atlas : vec![0; 32*32*8*8*3] }
    }

    /// Generate the atlas based on the tiles data and the tilemap in vram 
    pub fn generate(&mut self, addr: usize, tiles : &[Tile; 32*32], gpu_vram : &[u8; 8192]) {
        for y in 0..32 {
            for x in 0..32 {
                self.generate_single_tile(addr, x, y, tiles, gpu_vram);
            }
        }
    }

    pub fn generate_single_tile(&mut self, addr: usize, x: usize, y: usize, tiles : &[Tile; 32*32], gpu_vram : &[u8; 8192]) {
        let id = gpu_vram[addr + y*32 + x - 0x8000] as usize;
        self.blit_tile_to_map(x*8, y*8, id, tiles);
        //self.blit_tile_to_map(x*8, y*8, y*32+x, tiles); // Display the tilemap
    }

    /// Update all occurences of a tile, based on id, in the atlas.
    pub fn update_tiles_for_id(&mut self, addr : usize, id: usize, tiles : &[Tile; 32*32],  gpu_vram : &[u8; 8192]) {
        let id_u8 = id as u8;
        for y in 0..32 {
            for x in 0..32 {
                if gpu_vram[addr + y*32 + x - 0x8000] == id_u8 {
                    self.blit_tile_to_map(x*8, y*8, id, tiles);
                }
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

/// Main helper class
pub struct DrawHelper {
    // We only keep one active atlas in memory, though there are
    // technically 4 different atlases (tilemap1,tilemap2 | tiledata1, tiledata2),
    // but only one is used at a time. We modify the atlas to match
    // whatever combination is set.
    pub atlas : TileAtlas,
    pub tiles: [Tile; 32*32],
    pub background_palette : Palette,
    pub sprite_palette_1: Palette,
    pub sprite_palette_2: Palette,
    // Which tile data and tile map should we use? From lcd_control
    pub using_tiledata_1: bool, // Reversed from the actual bit for easier to read code
    pub using_tilemap_1: bool,
}

impl DrawHelper {
    pub fn new() -> DrawHelper {
        DrawHelper { atlas: TileAtlas::new(), 
            tiles: [Tile::new(); 32*32], background_palette : Palette::new(), sprite_palette_1 : Palette::new(),
            sprite_palette_2 : Palette::new(), using_tiledata_1 : false, using_tilemap_1: true, }
    }

    /// Generate the atlas from the active tilemap in VRAM
    pub fn generate_atlas(&mut self, gpu_vram: &[u8; 8192]) {
        if self.using_tilemap_1 {
            self.atlas.generate(0x9800, &self.tiles, gpu_vram);
        }
        else {
            self.atlas.generate(0x9C00, &self.tiles, gpu_vram);
        }
    }

    /// Generate the tiles from the active tiledata in VRAM
    pub fn generate_tiles(&mut self, gpu_vram: &[u8; 8192]) {
        if self.using_tiledata_1 {
            for id in 0..256 {
                self.tiles[id].generate(id, 0x8000, &self.background_palette, gpu_vram);
            }
        }
        else {
            for id in 0..256 {
                self.tiles[id].generate(id, 0x8800, &self.background_palette, gpu_vram);
            }
        }
    }

    /// Update the drawing acceleration structures based on a VRAM write
    pub fn update_by_vram_address(&mut self, address : usize, gpu_vram: &[u8; 8192]) {
        match address {
            0x8000 ..= 0x87FF => { self.update_tiledata_by_address(address, true, gpu_vram)} // Tile set #1: tiles 0-127
            0x8800 ..= 0x8FFF => { self.update_tiledata_by_address(address, true, gpu_vram);
                                self.update_tiledata_by_address(address, false, gpu_vram); } // Tile set #1: tiles 128-255, Tile set #0: tiles -1 to -128
            0x9000 ..= 0x97FF => { self.update_tiledata_by_address(address, false, gpu_vram)} // Tile set #0: tiles 0-127
            0x9800 ..= 0x9BFF => { self.update_tilemap_by_address(address, true, gpu_vram) } // Tile map #0
            0x9C00 ..= 0x9FFF => { self.update_tilemap_by_address(address, false, gpu_vram) } // Tile map #1
            0xFE00 ..= 0xFE9F => { } // OAM
            _ => { panic!("Invalid vram address")}
        }
    }

    /// Update the tilemap based on a VRAM write. 0x9800-0x9FFF
    /// As we only store one atlas
    pub fn update_tilemap_by_address(&mut self, address : usize, is_tilemap_1: bool, gpu_vram: &[u8; 8192]) {
        if self.using_tilemap_1 && is_tilemap_1 {
            let x = (address - 0x9800) % 32;
            let y = (address - 0x9800) / 32;
            self.atlas.generate_single_tile(0x9800, x, y, &self.tiles, gpu_vram);
        }
        else if !self.using_tilemap_1 && !is_tilemap_1 {
            let x = (address - 0x9C00) % 32;
            let y = (address - 0x9C00) / 32;
            self.atlas.generate_single_tile(0x9C00, x, y, &self.tiles, gpu_vram);
        }
    }

    /// Update the tilemap based on a VRAM write. 0x9800-0x9FFF
    pub fn update_tiledata_by_address(&mut self, address : usize, is_tiledata_1: bool, gpu_vram: &[u8; 8192])
    {
        if self.using_tiledata_1 && is_tiledata_1 {
            let id = (address - 0x8000) / 16;
            self.tiles[id].generate(id, 0x8000, &self.background_palette, gpu_vram);
            if self.using_tilemap_1 {
                self.atlas.update_tiles_for_id(0x9800, id, &self.tiles, gpu_vram);
            }
            else {
                self.atlas.update_tiles_for_id(0x9C00, id, &self.tiles, gpu_vram);
            }
        }
        else if !self.using_tiledata_1 && !is_tiledata_1 {
            let id = (address - 0x8800) / 16;
            self.tiles[id].generate(id, 0x8800, &self.background_palette, gpu_vram);
            if self.using_tilemap_1 {
                self.atlas.update_tiles_for_id(0x9800, id, &self.tiles, gpu_vram);
            }
            else {
                self.atlas.update_tiles_for_id(0x9C00, id, &self.tiles, gpu_vram);
            }
        }
    }

    pub fn update_lcd_control(&mut self, lcd_control : u8, gpu_vram: &[u8; 8192]) {
        if self.get_background_tile_data_select(lcd_control) != self.using_tiledata_1 {
            self.using_tiledata_1 = self.get_background_tile_data_select(lcd_control);
            self.generate_tiles(gpu_vram);
        }
        if self.get_background_tile_map_select(lcd_control) != self.using_tilemap_1 {
            self.using_tilemap_1 = self.get_background_tile_map_select(lcd_control);
            self.generate_atlas(gpu_vram);
        }
    }

    pub fn get_background_tile_map_select(&self, lcd_control : u8) -> bool {
        return lcd_control & 0b0001_0000 == 0b0001_0000;
    }

    /// Reversed from the actual bit here, as when the bit 4 is 0 means 8800-97FF 
    /// and 1 means 8000-8FFF, which is a confusing scheme.
    pub fn get_background_tile_data_select(&self, lcd_control : u8) -> bool {
        return !(lcd_control & 0b0000_1000 == 0b0000_1000);
    }
}

pub struct Palette {
    palette: [Color; 4]
}

/// Represents a 4 color palette
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