use std::convert::TryInto;

#[derive(Copy, Clone)]
enum MBCType {
    RomOnly,
    Mbc1,
    Mbc2,
    Mbc3,
    Mbc5,
}

pub struct Rom{
    rom_banks: Vec<[u8; 16384]>,
    ram_banks: Vec<[u8; 8192]>,
    current_rom_bank: u8,
    current_ram_bank: u8,
    external_ram_enabled: bool,
    ram_banking_mode: bool,
    pub filename: String,
    mbc_type: MBCType,
}

impl Rom {
    pub fn new() -> Rom
    {
        Rom { 
            rom_banks: Vec::new(), ram_banks: Vec::new(), current_rom_bank: 1, current_ram_bank: 0,
            external_ram_enabled: false, ram_banking_mode: false, filename: "".to_owned(), mbc_type: MBCType::RomOnly,
        }
    }

    pub fn read_byte(&self, addr : usize) -> u8 {
        match self.mbc_type {
            MBCType::RomOnly => { self.read_byte_rom_only(addr) } // Read-only memory
            MBCType::Mbc1    => { self.read_byte_mbc1(addr) } 
            MBCType::Mbc2    => { self.read_byte_mbc1(addr) } 
            MBCType::Mbc3    => { self.read_byte_mbc1(addr) } 
            MBCType::Mbc5    => { self.read_byte_mbc1(addr) }
        }
        //return self.rom_banks[self.current_bank_index][addr]
    }

    pub fn write_byte(&mut self, addr : usize, val: u8) {
        match self.mbc_type {
            MBCType::RomOnly => { } // Read-only memory
            MBCType::Mbc1    => { self.write_byte_mbc1(addr, val)}
            MBCType::Mbc2    => { }
            MBCType::Mbc3    => { }
            MBCType::Mbc5    => { }
        }
        //self.rom_banks[self.current_bank_index][addr as usize] = val;
    }
    
    pub fn read_from_file(&mut self, filename : &str) {
        let data = std::fs::read(filename);
        let data = match data {
            Ok(file) => file,
            Err(error) => panic!("Problem opening the rom file: {:?}", error),
        };
        self.filename = filename.to_owned();
        // Iterate over the banks and add them to the bank vector
        for bank in data.chunks(16384) {
            self.rom_banks.push(bank.try_into().unwrap());
        }
        let mbc_type = self.rom_banks[0][0x0147];
        self.mbc_type = match mbc_type {
            0x00 | 0x08 | 0x09 => MBCType::RomOnly,
            0x01 ..= 0x03 => MBCType::Mbc1,
            0x05 ..= 0x06 => MBCType::Mbc2,
            0x0F ..= 0x13 => MBCType::Mbc3,
            0x19 ..= 0x1E => MBCType::Mbc5,
            _ => { panic!("ROM error: Unsupported ROM type {}", mbc_type)}
        };

        let ram_size_byte = self.rom_banks[0][0x0148];
        let ram_bank_count = match ram_size_byte {
            0x00 ..= 0x02 => 1, // Everyone gets 1 bank for simplicity
            0x03 => 4,
            0x04 => 16,
            0x05 => 8,
            _ => 1,
        };

        for _i in 0..ram_bank_count {
            self.ram_banks.push([0; 8192]);
        }

        println!("Loaded ROM");
    }

    pub fn validate_checksum(&mut self) {

    }

    pub fn read_byte_rom_only(&self, addr : usize) -> u8 {
        match addr {
            0x0000 ..= 0x3FFF => { return self.rom_banks[0][addr]; }
            0x4000 ..= 0x7FFF => { return self.rom_banks[1][addr-0x4000]; }
            0xA000 ..= 0xBFFF => { return self.ram_banks[0][addr-0xA000]; }
            _ => { return 0; }
        }
    }

    pub fn read_byte_mbc1(&self, addr : usize) -> u8 {
        match addr {
            0x0000 ..= 0x3FFF => { return self.rom_banks[0][addr]; }
            0x4000 ..= 0x7FFF => { return self.rom_banks[self.current_rom_bank as usize][addr - 0x4000]}
            0xA000 ..= 0xBFFF => { return self.ram_banks[self.current_ram_bank as usize][addr - 0xA000]; }
            _ => { return 0; }
        }
    }

    pub fn write_byte_mbc1(&mut self, addr : usize, val: u8) {
        match addr {
            0x0000 ..= 0x1FFF => { self.external_ram_enabled = val & 0x0A == 0x0A } // RAM enable/disable
            0x2000 ..= 0x3FFF => { self.current_rom_bank = self.current_rom_bank & 0b1100_0000 | (if val == 0 {1} else {val})} // Switch ROM banks, lower 5 bits
            0x4000 ..= 0x5FFF => { 
                if !self.ram_banking_mode { // ROM banking mode
                    self.current_rom_bank = self.current_ram_bank & 0b0011_1111 | ((val & 0x03) << 5);
                }
                else {
                    self.current_ram_bank = val & 0x03;
                }
            }  // Switch ROM banks, upper  5bits
            0x6000 ..= 0x7FFF => { self.ram_banking_mode = val != 0} // ROM/RAM mode select
            0x4000 ..= 0x7FFF => { self.rom_banks[self.current_rom_bank as usize][addr - 0x4000] = val;} // Switch ROM banks, upper 2 bits
            0xA000 ..= 0xBFFF => { self.ram_banks[self.current_ram_bank as usize][addr - 0xA000] = val; }
            _ => {  }
        }
    }

}