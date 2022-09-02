#[derive(Copy, Clone)]
pub struct Bus {
    pub ram: [u8; 64 * 1024],
}

impl Bus {
    pub fn new() -> Bus {
        Bus {
            ram: [0; 64 * 1024],
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        if addr >= 0x0000 && addr <= 0xFFFF {
            self.ram[addr as usize] = data;
        } else {
            panic!("Invalid address: 0x{:X}", addr);
        }
    }

    pub fn read(&self, addr: u16, readOnly: bool) -> u8 {
        if addr >= 0x0000 && addr <= 0xFFFF {
            return self.ram[addr as usize];
        } else {
            return 0x00;
        }
    }
}