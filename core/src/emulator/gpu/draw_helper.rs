/// Contains acceleration structures for Drawing, based on
/// the tile memory in the GPU/PPU VRAM. When a byte is modified in the GPU,
/// it gets modified here as well, in a format that is better for rendering
/// 
/// The TileData class contains the Tile data located in 0x8000 - 0x97FF,
/// in a nice RGB format. All of the tiles are continually updated here,
/// even if the tiledata select only currently renders one of them
/// 
/// The TileMap combines the data from the TileData class and the
/// tilemap representation in the GPU to generate a 32x32 Tile Atlas.
/// 
/// The SpriteData represents the Sprite data located in OAM RAM, in
/// a nicer format. 
/// 
/// The SpriteMap uses the sprite 
/// The entire system is designed so that drawing a line can be done entirely with memcpys.

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

const COLOR_TRANSPARENT : Color = Color {r:255, g:255, b:255, a: 0};
const COLOR_WHITE: Color = Color {r:255, g:255, b:255, a: 255};
const COLOR_LIGHTGRAY: Color = Color {r:170, g:170, b:170, a: 255};
const COLOR_DARKGRAY: Color = Color {r:85, g:85, b:85, a:255};
const COLOR_BLACK: Color = Color {r:0, g:0, b:0, a:255};

/// Represents a 8x8 tile of Color
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Tile {
    pub pixels: [u8; 8*8*3],
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
    fn generate(&mut self, addr: usize, palette : &Palette, gpu_vram : &[u8; 8192]) {
        let data : &[u8] = &gpu_vram[addr..addr+16];
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

/// Represents the 384 tiles in GPU memory
pub struct TileData {
    // Represents the tile data stored between 0x8000-0x97FF
    pub tiles: [Tile; 384],
    pub tiledata_select : bool,
}

impl TileData {
    pub fn new() -> TileData {
        TileData {tiles : [Tile::new(); 384], tiledata_select: true}
    }

    /// Return a tile based on tile id, depending on the tiledata select
    pub fn get_tile(&self, tile_id : usize) -> &Tile {
        if self.tiledata_select { // Return tiles representing 0x8000-0x8FFF
            return &self.tiles[tile_id];
        }
        else { // Return tiles representing 0x8800-0x97FF
            return &self.tiles[tile_id + 128];
        }
    }

    /// Update the tile object which represents this memory address
    pub fn generate_tile(&mut self, data_address : usize, palette : &Palette, gpu_vram : &[u8; 8192]) {
        let id = (data_address - 0x8000) / 16;
        self.tiles[id].generate(id*16, palette, gpu_vram);
    }
}

/// Represents the 32x32 Tile Atlas, based on the Tilemap and Tiledata in the GPU/PPU
pub struct TileAtlas {
    pub atlas: Vec<u8>,
    pub tilemap_base_addr : usize,
}

impl TileAtlas {
    pub fn new(tilemap_base_addr : usize) -> TileAtlas {
        TileAtlas { atlas : vec![255; 32*32*8*8*3], tilemap_base_addr: tilemap_base_addr}
    }

    /// Generate the atlas based on the tiles data and the tilemap in vram 
    pub fn generate_all_tiles(&mut self, tiledata : &TileData, gpu_vram : &[u8; 8192]) {
        for y in 0..32 {
            for x in 0..32 {
                self.generate_single_tile(x, y, tiledata, gpu_vram);
            }
        }
    }

    /// Generate a single tile
    pub fn generate_single_tile(&mut self, x: usize, y: usize, tiledata : &TileData, gpu_vram : &[u8; 8192]) {
        let id = gpu_vram[self.tilemap_base_addr + y*32 + x - 0x8000] as usize;
        self.blit_tile_to_map(x*8, y*8, tiledata.get_tile(id));
        //self.blit_tile_to_map(x*8, y*8, tiledata.get_tile(y*32+x-1)); // Display the tilemap
    }

    /// Generate a single tile by address
    pub fn generate_single_tile_by_address(&mut self, addr: usize, tiledata : &TileData, gpu_vram : &[u8; 8192]) {
        let x = (addr - self.tilemap_base_addr) % 32;
        let y = (addr - self.tilemap_base_addr ) / 32;
        let id = gpu_vram[addr - 0x8000] as usize;
        self.blit_tile_to_map(x*8, y*8, tiledata.get_tile(id));
        //self.blit_tile_to_map(x*8, y*8, y*32+x, tiles); // Display the tilemap
    }

    /// Update all occurences of a tile, based on id, in the atlas.
    pub fn generate_all_tile_occurances(&mut self, addr : usize, id: usize, tile: &Tile, gpu_vram : &[u8; 8192]) {
        let id_u8 = id as u8;
        for y in 0..32 {
            for x in 0..32 {
                if gpu_vram[addr + y*32 + x - 0x8000] == id_u8 {
                    self.blit_tile_to_map(x*8, y*8, tile);
                }
            }
        }
    }

    pub fn blit_tile_to_map(&mut self, x: usize, y: usize, tile : &Tile) {
        let mut i: usize;
        for iy in 0..8 {
            i = (y+iy) * 256 * 3 + x*3;
            // Memcpy the row
            self.atlas[i..i+8*3].copy_from_slice(&tile.pixels[iy*8*3..iy*8*3+8*3]);
        }
    }
}

/// Represents a Sprite, which is located in OAM memory
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Sprite {
    pub x : usize,
    pub y : usize,
    pub tile_id : usize,
    pub below_background : bool,
    pub flip_y : bool,
    pub flip_x : bool,
    pub palette_select : bool,
}

impl Sprite {
    pub fn new() -> Sprite {
        Sprite { x: 0, y: 0, tile_id: 0, below_background : false, 
                flip_y: false, flip_x: false, palette_select: false}
    }

    pub fn generate_by_id(&mut self, id: usize, oam_ram : &[u8; 160]) {
        let base_addr = id*4;
        self.y = oam_ram[base_addr + 0] as usize;
        self.x = oam_ram[base_addr + 1] as usize;
        self.tile_id = oam_ram[base_addr + 2] as usize;
        self.set_options(oam_ram[base_addr + 3]);
    }

    fn set_options(&mut self, options : u8) {
        self.below_background = options & 0b1000_0000 == 0b1000_0000;
        self.flip_y = options & 0b0100_0000 == 0b0100_0000;
        self.flip_x = options & 0b0010_0000 == 0b0010_0000;
        self.palette_select = options & 0b0001_0000 == 0b0001_0000;
    }
}

pub struct SpriteData {
    pub sprites: [Sprite; 40],
}

impl SpriteData {
    pub fn new() -> SpriteData {
        SpriteData { sprites: [Sprite::new(); 40]}
    }

    pub fn update_by_address(&mut self, addr : usize, oam_ram : &[u8; 160]) {
        let id = (addr - 0xFE00) / 4;
        self.sprites[id].generate_by_id(id, oam_ram);
    }

    pub fn get_sprite(&self, id : usize) -> &Sprite {
        return &self.sprites[id];
    }
}

/// Main helper class
pub struct DrawHelper {
    // We only keep one active atlas in memory, though there are
    // technically 4 different atlases (tilemap1,tilemap2 | tiledata1, tiledata2),
    // but only one is used at a time. We modify the atlas to match
    // whatever combination is set.
    pub bg_atlas1 : TileAtlas, // Tilemap 1 atlas
    pub bg_atlas2 : TileAtlas, // Tilemap 2 atlas
    pub tile_data : TileData,

    pub sprite_data: SpriteData,

    pub background_palette : Palette,
    pub sprite_palette_1: Palette,
    pub sprite_palette_2: Palette,
    // Which tile data and tile map should we use? From lcd_control
    pub tiledata_select: bool, // Is bit set?
    pub bg_tilemap_select: bool, // Is bit set?
    pub window_tilemap_select: bool, // Is bit set?
}

impl DrawHelper {
    pub fn new() -> DrawHelper {
        DrawHelper { 
            bg_atlas1: TileAtlas::new(0x9C00), 
            bg_atlas2: TileAtlas::new(0x9800), 
            tile_data: TileData::new(), 
            sprite_data: SpriteData::new(),
            background_palette : Palette::new_bg(), 
            sprite_palette_1 : Palette::new_sprite(),
            sprite_palette_2 : Palette::new_sprite(), 
            tiledata_select : false, 
            window_tilemap_select: false, 
            bg_tilemap_select: true }
    }

    /// Generate the atlas from the active tilemap in VRAM
    pub fn generate_atlas(&mut self, gpu_vram: &[u8; 8192]) {
        if self.window_tilemap_select || self.bg_tilemap_select{
            self.bg_atlas1.generate_all_tiles(&mut self.tile_data, gpu_vram);
        }
        else if !self.window_tilemap_select || !self.bg_tilemap_select {
            self.bg_atlas2.generate_all_tiles(&mut self.tile_data, gpu_vram);
        }
    }

    /// Generate the tiles from the active tiledata in VRAM
    pub fn generate_tiles(&mut self, gpu_vram: &[u8; 8192]) {
        for id in 0..384 {
            self.tile_data.generate_tile(0x8000+id*16, &self.background_palette, gpu_vram);
        }
    }

    pub fn generate_sprites(&mut self, oam_ram: &[u8; 160]) {
        for i in 0..160 {
            self.update_sprites_by_adress(0xFE00+i, oam_ram);
        }
    }

    /// Update the drawing acceleration structures based on a VRAM write
    pub fn update_by_vram_address(&mut self, address : usize, gpu_vram: &[u8; 8192], oam_ram : &[u8; 160]) {
        match address {
            0x8000 ..= 0x87FF => { self.update_tiledata_by_address(address, true, gpu_vram)} // Tile set #1: tiles 0-127
            0x8800 ..= 0x8FFF => { self.update_tiledata_by_address(address, true, gpu_vram);
                                self.update_tiledata_by_address(address, false, gpu_vram); } // Tile set #1: tiles 128-255, Tile set #0: tiles -1 to -128
            0x9000 ..= 0x97FF => { self.update_tiledata_by_address(address, false, gpu_vram)} // Tile set #0: tiles 0-127
            0x9800 ..= 0x9BFF => { self.update_tilemap_by_address(address, true, gpu_vram) } // Tile map #0
            0x9C00 ..= 0x9FFF => { self.update_tilemap_by_address(address, false, gpu_vram) } // Tile map #1
            0xFE00 ..= 0xFE9F => { self.update_sprites_by_adress(address, oam_ram)} // OAM
            _ => { panic!("Invalid vram address")}
        }
    }

    /// Update the tilemap based on a VRAM write. 0x9800-0x9FFF
    /// As we only store one atlas
    pub fn update_tilemap_by_address(&mut self, address : usize, is_tilemap_1: bool, gpu_vram: &[u8; 8192]) {
        // Only modify atlas if we are currently using this tilemap
        if !is_tilemap_1 && (self.bg_tilemap_select || self.window_tilemap_select) {
            self.bg_atlas1.generate_single_tile_by_address(address, &mut self.tile_data, gpu_vram);
        } // Same here
        else if is_tilemap_1 && (!self.bg_tilemap_select || !self.window_tilemap_select) {
            self.bg_atlas2.generate_single_tile_by_address(address, &mut self.tile_data, gpu_vram);
        }
    }

    pub fn update_sprites_by_adress(&mut self, address : usize, oam_ram: &[u8; 160]) {
        self.sprite_data.update_by_address(address, oam_ram);
    }

    pub fn get_background_atlas(&self) -> &TileAtlas {
        if self.bg_tilemap_select {
            return &self.bg_atlas1;
        }
        else {
            return &self.bg_atlas2;
        }

    }

    pub fn get_window_atlas(&self) -> &TileAtlas {
        if self.window_tilemap_select {
            return &self.bg_atlas1;
        }
        else {
            return &self.bg_atlas2;
        }
    }

    /// Update the tilemap based on a VRAM write. 0x9800-0x9FFF
    pub fn update_tiledata_by_address(&mut self, address : usize, is_tiledata_1: bool, gpu_vram: &[u8; 8192])
    {
        self.tile_data.generate_tile(address, &self.background_palette, gpu_vram);
        let id = (address - 0x8000) / 16;
        if self.bg_tilemap_select || self.window_tilemap_select {
            self.bg_atlas1.generate_all_tile_occurances(0x9C00, id, self.tile_data.get_tile(id), gpu_vram);
        }
        else if !self.bg_tilemap_select || self.window_tilemap_select {
            self.bg_atlas2.generate_all_tile_occurances(0x9C00, id, self.tile_data.get_tile(id), gpu_vram);
        }
    }

    //
    pub fn update_lcd_control(&mut self, lcd_control : u8, gpu_vram: &[u8; 8192]) {
        if self.get_tile_data_select(lcd_control) != self.tiledata_select {
            self.tiledata_select = self.get_tile_data_select(lcd_control);
            self.tile_data.tiledata_select = self.tiledata_select;
            // Tile data changed, we need to recalculate the entire atlas
            self.generate_atlas(gpu_vram);
        }
        if self.get_background_tile_map_select(lcd_control) != self.bg_tilemap_select {
            self.bg_tilemap_select = self.get_background_tile_map_select(lcd_control);
        }
    }

    // Helpers for accessing LCD_Control
    pub fn get_window_tile_map_select(&self, lcd_control : u8) -> bool {
        return lcd_control & 0b0100_0000 == 0b0100_0000;
    }

    pub fn get_background_tile_map_select(&self, lcd_control : u8) -> bool {
        return lcd_control & 0b0000_1000 == 0b0000_1000;
    }

    pub fn get_tile_data_select(&self, lcd_control : u8) -> bool {
        return lcd_control & 0b0001_0000 == 0b0001_0000;
    }
}


pub struct Palette {
    palette: [Color; 4],
    map: [Color; 4]
}

/// Represents a 4 color palette
impl Palette {
    fn new_bg() -> Palette {
        return Palette { 
            palette: [COLOR_WHITE, COLOR_LIGHTGRAY, COLOR_DARKGRAY, COLOR_BLACK],
            map: [COLOR_WHITE, COLOR_LIGHTGRAY, COLOR_DARKGRAY, COLOR_BLACK],
        }
    }

    fn new_sprite() -> Palette {
        return Palette { 
            palette: [COLOR_TRANSPARENT, COLOR_LIGHTGRAY, COLOR_DARKGRAY, COLOR_BLACK],
            map: [COLOR_TRANSPARENT, COLOR_LIGHTGRAY, COLOR_DARKGRAY, COLOR_BLACK],
        }
    }

    pub fn update(&mut self, palette_flag: u8) {
        self.palette[0] = self.get_color_from_bits(palette_flag & 0b0000_0011);
        self.palette[1] = self.get_color_from_bits((palette_flag & 0b0000_1100) >> 2);
        self.palette[2] = self.get_color_from_bits((palette_flag & 0b0011_0000) >> 4);
        self.palette[3] = self.get_color_from_bits((palette_flag & 0b1100_0000) >> 6);
    }

    fn get_color_from_bits(&mut self, color_val: u8) -> Color {
        return self.map[color_val as usize];
    }
}