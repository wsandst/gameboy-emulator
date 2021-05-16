#![allow(dead_code)]
use std::convert::TryInto;

use serde::{Serialize, Deserialize};
use serde_big_array::BigArray;

const KB : usize = 1024;

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
enum MBCType {
    RomOnly,
    Mbc1,
    Mbc2,
    Mbc3,
    Mbc5,
}

#[derive(Serialize, Deserialize)]
pub struct Rom{
    rom_banks: Vec<Vec<u8>>,
    ram_banks: Vec<Vec<u8>>,
    current_rom_bank: u8,
    current_ram_bank: u8,
    external_ram_enabled: bool,
    ram_banking_mode: bool,
    pub filename: String,
    mbc_type: MBCType,
    pub using_boot_rom: bool,
    #[serde(with = "BigArray")]
    boot_rom: [u8; 256],
}

impl Rom {
    pub fn new() -> Rom
    {
        Rom { 
            rom_banks: Vec::new(), 
            ram_banks: Vec::new(), 
            current_rom_bank: 1, 
            current_ram_bank: 0,
            external_ram_enabled: false, 
            ram_banking_mode: false, 
            filename: "".to_owned(), 
            mbc_type: MBCType::RomOnly,
            using_boot_rom: false,
            boot_rom: [0; 256],
        }
    }

    pub fn load_from_file(&mut self, filename: &str) {
        let data = std::fs::read(filename);
        let data = match data {
            Ok(file) => file,
            Err(error) => panic!("Problem opening the rom file: {:?}", error),
        };
        self.filename = filename.to_owned();
        self.load_from_data(&data);
    }

    pub fn load_from_data(&mut self, data: &Vec<u8>) {
        // Iterate over the banks and add them to the bank vector
        for bank in data.chunks(16*KB) {
            self.rom_banks.push(bank.try_into().unwrap());
        }
        let mbc_type = self.rom_banks[0][0x0147];
        self.mbc_type = match mbc_type {
            0x00 | 0x08 | 0x09 => MBCType::RomOnly,
            0x01 ..= 0x03 => MBCType::Mbc1,
            //0x05 ..= 0x06 => MBCType::Mbc2,
            0x0F ..= 0x13 => MBCType::Mbc3,
            //0x19 ..= 0x1E => MBCType::Mbc5,
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
            self.ram_banks.push(vec![0; 8192]);
        }

        if !self.is_header_checksum_valid() {
            panic!("ROM header checksum invalid!")
        }

        println!("Loaded ROM");
    }

    pub fn load_bootrom_from_file(&mut self, filename: &str) {
        let data = std::fs::read(filename);
        let data = match data {
            Ok(file) => file,
            Err(error) => panic!("Problem opening the bootrom file: {:?}", error),
        };
        if data.len() != 256 {
            panic!("Loaded bootrom {} file was {} bytes instead of expected 256 bytes", filename, data.len());
        }
        self.filename = filename.to_owned();
        self.load_bootrom_from_data(&data);
    }

    pub fn load_bootrom_from_data(&mut self, data: &Vec<u8>) {
        self.boot_rom.clone_from_slice(&data);
    }

    pub fn read_byte(&self, addr : usize) -> u8 {
        if self.using_boot_rom && addr < 256 { // Boot rom read
            return self.boot_rom[addr];
        }
        match self.mbc_type {
            MBCType::RomOnly => { self.read_byte_rom_only(addr) } // Read-only memory
            MBCType::Mbc1    => { self.read_byte_mbc1(addr) } 
            MBCType::Mbc2    => { panic!("MBC2 is not implemented") } 
            MBCType::Mbc3    => { self.read_byte_mbc3(addr) } 
            MBCType::Mbc5    => { panic!("MBC5 is not implemented")}
        }
        //return self.rom_banks[self.current_bank_index][addr]
    }

    pub fn write_byte(&mut self, addr : usize, val: u8) {
        match self.mbc_type {
            MBCType::RomOnly => { } // Read-only memory
            MBCType::Mbc1    => { self.write_byte_mbc1(addr, val)}
            MBCType::Mbc2    => { panic!("MBC2 is not implemented") }
            MBCType::Mbc3    => { self.write_byte_mbc3(addr, val)}
            MBCType::Mbc5    => { panic!("MBC5 is not implemented") }
        }
        //self.rom_banks[self.current_bank_index][addr as usize] = val;
    }

    pub fn is_header_checksum_valid(&mut self) -> bool {
        // x=0:FOR i=0134h TO 014Ch:x=x-MEM[i]-1:NEXT
        let mut x: u8 = 0;
        for i in 0x0134..0x014D {
            x = x.wrapping_sub(self.rom_banks[0][i]).wrapping_sub(1);
        }
        return self.rom_banks[0][0x14D] == x;
    }

    pub fn read_byte_rom_only(&self, addr : usize) -> u8 {
        match addr {
            0x0000 ..= 0x3FFF => { return self.rom_banks[0][addr]; }
            0x4000 ..= 0x7FFF => { return self.rom_banks[1][addr-0x4000]; }
            0xA000 ..= 0xBFFF => { return self.ram_banks[0][addr-0xA000]; }
            _ => { return 0; }
        }
    }

    // MBC1
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
                    self.current_ram_bank = 0;
                }
                else {
                    self.current_ram_bank = val & 0x03;
                }
            }  // Switch ROM banks, upper  5bits
            0x6000 ..= 0x7FFF => { self.ram_banking_mode = val != 0} // ROM/RAM mode select
            0xA000 ..= 0xBFFF => { self.ram_banks[self.current_ram_bank as usize][addr - 0xA000] = val; }
            _ => {  }
        }
    }

    // MBC3
    pub fn read_byte_mbc3(&self, addr : usize) -> u8 {
        match addr {
            0x0000 ..= 0x3FFF => { return self.rom_banks[0][addr]; } // Fine
            0x4000 ..= 0x7FFF => { return self.rom_banks[self.current_rom_bank as usize][addr - 0x4000]}
            0xA000 ..= 0xBFFF => { 
                if self.current_ram_bank < 8 {
                    return self.ram_banks[self.current_ram_bank as usize][addr - 0xA000]; 
                }
                else {
                    return 0; // Temporary RTC fix
                }
            }
            _ => { return 0; }
        }
    }

    pub fn write_byte_mbc3(&mut self, addr : usize, val: u8) {
        match addr {
            0x0000 ..= 0x1FFF => { self.external_ram_enabled = val & 0x0A == 0x0A } // RAM enable/disable
            0x2000 ..= 0x3FFF => { self.current_rom_bank = self.current_rom_bank & 0b1100_0000 | (if val == 0 {1} else {val})} // Switch ROM banks, lower 5 bits
            0x4000 ..= 0x5FFF => { 
                self.current_ram_bank = self.current_ram_bank & 0b0011_1111 | ((val & 0x03) << 5);
                if self.current_ram_bank > 8 {
                    //panic!("MBC3 Real-time Clock is not implemented");
                }
            }  // Switch ROM banks, upper  5bits
            0x6000 ..= 0x7FFF => { }//panic!("MBC3 Real-time Clock is not implemented")} // RTC write
            0xA000 ..= 0xBFFF => { 
                if self.current_ram_bank < 8 { 
                    self.ram_banks[self.current_ram_bank as usize][addr - 0xA000] = val; 
                }
            }
            _ => {  }
        }
    }

    /// Return a slice of ROM memory, used for DMA transfers
    pub fn read_mem_slice(&self, start_addr : usize, end_addr : usize) -> &[u8] {
        match start_addr {
            0x0000 ..= 0x3FFF => &self.rom_banks[0][start_addr..end_addr],
            0x4000 ..= 0x7FFF => &self.rom_banks[1][start_addr-0x4000..end_addr-0x4000],
            0xA000 ..= 0xBFFF => &self.ram_banks[0][start_addr-0xA000..end_addr-0xA000],
            _ => { panic!("Invalid ROM memory address")}
        }
    }
}

#[cfg(test)]
mod test
{
    use super::Rom;
    use super::MBCType;

    #[test]
    fn mbc1()
    {
        let mut rom = Rom::new();
        rom.load_from_file("../roms/blargg/cpu_instrs.gb");

        assert_eq!(rom.mbc_type, MBCType::Mbc1);
        assert_eq!(rom.is_header_checksum_valid(), true);

        // ROM switching
        rom.rom_banks[3][0] = 42; // Read-only, but doing a write for testing
        rom.write_byte(0x2000, 0x3); // Switching to bank 3
        assert_eq!(rom.read_byte(0x4000), 42); // Check if we can find our value
    }
}