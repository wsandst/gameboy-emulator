// Temporary
#![allow(dead_code)]

pub mod draw_helper;

#[derive(Copy, Clone, PartialEq, Debug)]
enum LCDMode {
    HBlankPeriod,
    VBlankPeriod,
    UsingOAMPeriod,
    UsingVRAMPeriod,
}

/// Represents the PPU/GPU of a Gameboy/Gameboy Color.
pub struct GPU {
    pub video_ram: [u8; 8192], // 8kb, 0x8000 - 0x9FFF
    pub oam_ram: [u8; 160], // 160 bytes, 0xFE00 - 0xFE9F

    // GPU/PPU Device Control memory
    pub lcd_control: u8, // 0xFF40 LCDC
    pub lcd_stat: u8, // 0xFF41 STAT

    pub scroll_y: u8, // 0xFF42 Scroll Y (Background upper left pos)
    pub scroll_x: u8, // 0xFF43 Scroll X (Background upper left pos)
    pub ly: u8, // xFF44, Current Vertical Line
    pub lyc: u8, // 0xFF45, Compared with ly, if same then STAT interrupt
    pub window_y: u8, // 0xFF4A Window Y (Window upper left pos)
    pub window_x: u8, // 0xFF4B Window X (Window upper left pos)

    pub oam_dma_transfer: u8, //0xFF46

    pub background_palette: u8, // 0xFF47 BGP
    pub sprite_palette_1: u8, // 0xFF48
    pub sprite_palette_2: u8, // 0xFF49

    clock_cycles: usize,
    pub scanline_draw_requested: bool,
    pub screen_draw_requested: bool,

    // Interrupt related
    // STAT Interrupts enables, from lcd_status 0xFF41
    pub stat_hblank_inter_enable: bool,
    pub stat_vblank_inter_enable: bool,
    pub stat_oam_inter_enable: bool,
    pub stat_lyc_inter_enable: bool,

    pub vblank_interrupt_requested: bool,
    pub stat_interrupt_requested: bool,

    // Drawing helpers
    pub state_modified: bool,

    pub draw_helper : draw_helper::DrawHelper,
}

impl GPU {
    pub fn new() -> GPU {
        GPU { 
            video_ram: [0; 8192], 
            oam_ram: [0; 160], 
            lcd_control: 0, 
            lcd_stat: 0, 
            scroll_y: 0, 
            scroll_x: 0, 
            ly: 0, 
            lyc: 0,
            window_y: 0,
            window_x: 0, 
            oam_dma_transfer: 0, 
            background_palette: 0,
            sprite_palette_1: 0, 
            sprite_palette_2: 0, 
            clock_cycles: 0, 
            scanline_draw_requested: false, 
            screen_draw_requested: false, 
            vblank_interrupt_requested: false, 
            stat_interrupt_requested: false,
            stat_hblank_inter_enable: false,
            stat_vblank_inter_enable: false,
            stat_oam_inter_enable: false,
            stat_lyc_inter_enable: false,
            state_modified: false,
            draw_helper : draw_helper::DrawHelper::new()
        }
    }

    pub fn read_byte(&self, address: usize) -> u8 {
        match address {
            0x8000 ..= 0x9FFF => { return self.video_ram[address - 0x8000] }
            0xFE00 ..= 0xFE9F => { return self.oam_ram[address - 0xFE00] }
            // Device control addresses
            0xFF40 => { return self.lcd_control; }
            0xFF41 => { return self.lcd_stat; }
            0xFF42 => { return self.scroll_y; }
            0xFF43 => { return self.scroll_x; }
            0xFF44 => { return self.ly; }
            0xFF45 => { return self.lyc; }
            0xFF46 => { return self.oam_dma_transfer; }
            0xFF47 => { return self.background_palette; }
            0xFF48 => { return self.sprite_palette_1; }
            0xFF49 => { return self.sprite_palette_2; }
            0xFF4A => { return self.window_y; }
            0xFF4B => { return self.window_x; }
            _ => { panic!("Illegal memory access at addr {} in GPU", address)}
        }
    }

    pub fn write_byte(&mut self, address: usize, value: u8) {
        self.state_modified = true;
        match address {
            0x8000 ..= 0x9FFF => { 
                self.video_ram[address - 0x8000] = value; 
                self.draw_helper.update_by_vram_address(address, &self.video_ram, &self.oam_ram); 
            }
            0xFE00 ..= 0xFE9F => { 
                self.oam_ram[address - 0xFE00] = value; 
                self.draw_helper.update_by_vram_address(address, &self.video_ram, &self.oam_ram)
            }

            // Device control addresses
            0xFF40 => { self.set_lcd_control(value); }
            0xFF41 => { self.set_lcd_stat(value); }
            0xFF42 => { self.scroll_y = value; }
            0xFF43 => { self.scroll_x = value; }
            0xFF44 => { self.ly = value; }
            0xFF45 => { self.lyc = value; }
            0xFF46 => { self.oam_dma_transfer = value; }
            0xFF47 => { self.background_palette = value; self.update_palettes(); }
            0xFF48 => { self.sprite_palette_1 = value; self.update_palettes(); }
            0xFF49 => { self.sprite_palette_2 = value; self.update_palettes(); }
            0xFF4A => { self.window_y = value; }
            0xFF4B => { self.window_x = value; }
            _ => { panic!("Illegal memory write at addr {} in GPU", address) }
        }
    }

    pub fn cycle(&mut self, cycles : usize) {
        if !self.get_lcd_display_enable() {
            return; // Display disabled, do not cycle
        } 

        self.clock_cycles += cycles;

        match self.get_lcd_mode_flag() {

            // Horizontal blank period, Scanline not active, 204 cycles
            LCDMode::HBlankPeriod => {
                if self.clock_cycles >= 204 {
                    self.clock_cycles -= 204;
                    self.set_lcd_mode_flag(LCDMode::UsingVRAMPeriod);
                    self.ly += 1;
                    self.check_for_lyc_interrupt();

                    if self.ly > 143 {
                        // Enter vblank
                        self.set_lcd_mode_flag(LCDMode::VBlankPeriod);
                        self.check_for_stat_interrupt();
                        // Render entire frame
                        self.screen_draw_requested = true;
                        self.vblank_interrupt_requested = true;
                    }
                    else {
                        self.set_lcd_mode_flag(LCDMode::UsingOAMPeriod);
                        self.check_for_stat_interrupt();
                    }
                }
            }

            // Vertical blank period, Scanline not active, 10 lines * 456 cycles
            LCDMode::VBlankPeriod => {
                if self.clock_cycles >= 456 {
                    self.clock_cycles -= 456;
                    self.ly += 1;
                    self.check_for_lyc_interrupt();
                    if self.ly == 153 { // After 10 lines of VBlank, start drawing again
                        self.ly = 0;
                        self.set_lcd_mode_flag(LCDMode::UsingOAMPeriod);
                        self.check_for_stat_interrupt();
                    }
                }
            }

            // Read from OAM, Scanline Active, 80 cycles
            LCDMode::UsingOAMPeriod => { 
                if self.clock_cycles >= 80 {
                    self.clock_cycles = self.clock_cycles - 80;
                    self.set_lcd_mode_flag(LCDMode::UsingVRAMPeriod);
                }
            }

            // Read from VRAM, Scanline Active, 172 cycles
            LCDMode::UsingVRAMPeriod => {
                if self.clock_cycles >= 172 {
                    self.clock_cycles -= 172;
                    self.set_lcd_mode_flag(LCDMode::HBlankPeriod);
                    self.check_for_stat_interrupt();
                    self.scanline_draw_requested = true;
                }
            }
        }

    }


    fn get_lcd_mode_flag(&self) -> LCDMode {
        match self.lcd_stat & 0b0000_0011 {
            0 => LCDMode::HBlankPeriod,
            1 => LCDMode::VBlankPeriod,
            2 => LCDMode::UsingOAMPeriod,
            3 => LCDMode::UsingVRAMPeriod,
            _ => panic!("Error: Impossible LCDMode detected"),
        }
    }

    fn get_lcd_display_enable(&self) -> bool {
        return self.lcd_control & 0b1000_0000 == 0b1000_0000; // Bit 7 of LCDC
    }

    pub fn get_bg_priority_lcdc0(&self) -> bool {
        return self.lcd_control & 0b0000_0001 == 0b0000_0001; // Bit 0 of LCDC
    }

    pub fn get_window_enable(&self) -> bool {
        return self.lcd_control & 0b0010_0000 == 0b0010_0000; // Bit 5 of LCDC
    }

    pub fn get_sprite_enable(&self) -> bool {
        return self.lcd_control & 0b0000_0010 == 0b0000_0010; // Bit 1 of LCDC
    }

    pub fn set_lcd_control(&mut self, lcd_control : u8) {
        self.lcd_control = lcd_control;
        if !self.get_lcd_display_enable() { // Display disabled, reset GPU
            self.ly = 0;
            self.clock_cycles = 0;
            self.set_lcd_mode_flag(LCDMode::HBlankPeriod);
        }
        self.draw_helper.update_lcd_control(lcd_control, &self.video_ram);
    }

    pub fn set_lcd_stat(&mut self, lcd_stat : u8) {
        // Bit 6, lyc == ly
        self.stat_lyc_inter_enable = 0b0100_0000 & lcd_stat == 0b01000_000;
        // Bit 5, OAMPeriod (and VBLANK)
        self.stat_oam_inter_enable = 0b0010_0000 & lcd_stat == 0b0010_0000;
        // Bit 4, VBlank
        self.stat_vblank_inter_enable = 0b0001_0000 & lcd_stat == 0b0001_0000;
        // Bit 3, HBlank
        self.stat_hblank_inter_enable = 0b0000_1000 & lcd_stat == 0b0000_1000;
        self.lcd_stat = lcd_stat;
    }

    fn set_lcd_mode_flag(&mut self, mode : LCDMode) {
        let f = match mode {
            LCDMode::HBlankPeriod => 0,
            LCDMode::VBlankPeriod => 1,
            LCDMode::UsingOAMPeriod => 2,
            LCDMode::UsingVRAMPeriod => 3,
        };
        self.lcd_stat = self.lcd_stat & 0b1111_1100 | f;
    }

    fn check_for_stat_interrupt(&mut self) {
        self.stat_interrupt_requested = match self.get_lcd_mode_flag() {
            LCDMode::UsingOAMPeriod | LCDMode::VBlankPeriod if self.stat_oam_inter_enable => true,
            LCDMode::VBlankPeriod if self.stat_vblank_inter_enable => true,
            LCDMode::HBlankPeriod if self.stat_hblank_inter_enable => true,
            _ => false
        }
    }

    fn check_for_lyc_interrupt(&mut self) {
        if self.stat_lyc_inter_enable && self.lyc == self.ly {
            self.stat_interrupt_requested = true;
        }
    }

    fn update_palettes(&mut self) {
        self.draw_helper.background_palette.update(self.background_palette);
        self.draw_helper.sprite_palette_1.update(self.sprite_palette_1);
        self.draw_helper.sprite_palette_2.update(self.sprite_palette_2);
    }

    pub fn should_draw_scanline(&self) -> bool {
        return self.scanline_draw_requested && self.get_lcd_display_enable();
    }
}

#[cfg(test)]
mod test
{
    use super::super::memory;
    use super::LCDMode;

    #[test]
    fn gpu_mode_switching()
    {
        let mut mem = memory::Memory::new();
        mem.write_byte(0xFF40, 0b1000_0000); // Enable LCD, bit 7
        assert_eq!(mem.gpu.get_lcd_mode_flag(), LCDMode::HBlankPeriod); 
        mem.gpu.cycle(204);
        assert_eq!(mem.gpu.get_lcd_mode_flag(), LCDMode::UsingOAMPeriod);
        assert_eq!(mem.gpu.ly, 1);
        mem.gpu.cycle(81); // One extra to carry over
        assert_eq!(mem.gpu.get_lcd_mode_flag(), LCDMode::UsingVRAMPeriod);
        mem.gpu.cycle(171); // One less to adjust for previous
        assert_eq!(mem.gpu.get_lcd_mode_flag(), LCDMode::HBlankPeriod);
        for _i in 0..8*143 {
            mem.gpu.cycle(57); // Should be at VBlank now
        }
        assert_eq!(mem.gpu.get_lcd_mode_flag(), LCDMode::VBlankPeriod);
        assert_eq!(mem.gpu.ly, 144);
    }
}