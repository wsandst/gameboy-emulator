
/// Represents the GPU/PPU of a Gameboy/Gameboy Color.
pub struct GPU {
    video_ram: [u8; 8192], // 8kb, 0x8000 - 0x9FFF
    oam_ram: [u8; 160], // 160 bytes, 0xFE00 - 0xFE9F
}

impl GPU {
    pub fn new() -> GPU {
        GPU { video_ram: [0; 8192], oam_ram: [0; 160] }
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