

pub struct GPU {
    video_ram: [u8; 8192], // 8kb, 0x8000 - 0x9FFF
    oam_ram: [u8; 160], // 160 bytes, 0xFE00 - 0xFE9F
}

impl GPU {
    pub fn new() {
        
    }
}