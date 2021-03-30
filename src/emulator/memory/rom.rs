use std::convert::TryInto;

pub struct Rom{
    banks: Vec<[u8; 32768]>,
    current_bank_index: usize,
    pub filename: String,
}

impl Rom {
    pub fn new() -> Rom
    {
        Rom { 
            banks: Vec::new(), current_bank_index: 0, filename: "".to_owned()
        }
    }

    pub fn read_byte(&self, addr : usize) -> u8 {
        return self.banks[self.current_bank_index][addr]
    }

    pub fn write_byte(&mut self, addr : usize, val: u8) {
        self.banks[self.current_bank_index][addr as usize] = val;
    }
    
    pub fn read_from_file(&mut self, filename : &str) {
        let data = std::fs::read(filename);
        let data = match data {
            Ok(file) => file,
            Err(error) => panic!("Problem opening the rom file: {:?}", error),
        };
        self.filename = filename.to_owned();
        // Iterate over the banks and add them to the bank vector
        for bank in data.chunks(32768) {
            self.banks.push(bank.try_into().unwrap());
        }
    }
}