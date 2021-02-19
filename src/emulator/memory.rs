
const WRAM_SIZE: usize = 32768; //32 kb
const VRAM_SIZE: usize = 8192; // 8 kb
const HRAM_SIZE: usize = 32768;

pub struct Memory
{
    wram: [u8; WRAM_SIZE],
}
