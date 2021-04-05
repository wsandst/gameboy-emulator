
/// Represents the GPU/PPU of a Gameboy/Gameboy Color.
pub struct GPU {
    video_ram: [u8; 8192], // 8kb, 0x8000 - 0x9FFF
    oam_ram: [u8; 160], // 160 bytes, 0xFE00 - 0xFE9F

    pub lcd_control: u8, // 0xFF40 LCDC
    pub lcd_status: u8, // 0xFF41 STAT

    pub scroll_y: u8, // 0xFF42 Scroll Y (Background upper left pos)
    pub scroll_x: u8, // 0xFF43 Scroll X (Background upper left pos)
    pub ly: u8, // xFF44, Current Vertical Line
    pub lyc: u8, // 0xFF45, Compared with ly, if same then STAT interrupt
    pub window_y: u8, // 0xFF4A Window Y (Window upper left pos)
    pub window_x: u8, // 0xFF4B Window X (Window upper left pos)

    pub oam_transfer_request: u8, //0xFF46

    pub background_palette: u8, // 0xFF47 BGP
    pub sprite_palette_0: u8, // 0xFF48
    pub sprite_palette_1: u8, // 0xFF49
}

impl GPU {
    pub fn new() -> GPU {
        GPU { video_ram: [0; 8192], oam_ram: [0; 160], 
            lcd_control: 0, lcd_status: 0, scroll_y: 0, scroll_x: 0, ly: 0, lyc: 0,
            window_y: 0, window_x: 0, oam_transfer_request: 0, background_palette: 0,
            sprite_palette_0: 0, sprite_palette_1: 0 }
    }

    pub fn read_byte(&self, address: usize) -> u8 {
        match address {
            0x8000 ..= 0x9FFF => { return self.video_ram[address - 0x8000] }
            0xFE00 ..= 0xFE9F => { return self.oam_ram[address - 0xFE00] }
            _ => { panic!("Illegal memory access at addr {} in GPU", address)}
        }
    }

    pub fn write_byte(&mut self, address: usize, value: u8) {
        match address {
            0x8000 ..= 0x9FFF => { self.video_ram[address - 0x8000] = value; }
            0xFE00 ..= 0xFE9F => { self.oam_ram[address - 0xFE00] = value; }
            _ => { panic!("Illegal memory write at addr {} in GPU", address) }
        }
    }
}