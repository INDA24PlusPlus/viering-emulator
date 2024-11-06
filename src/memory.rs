use crate::read_byte;

pub struct Memory {
    data: [u16; 0x10000],
}

impl Memory {
    pub fn new() -> Self {
        Self { data: [0; 0x10000] }
    }

    pub fn read(&self, addr: u16) -> u16 {
        if addr == 0xfe00 {
            return 0x8000;
        }
        if addr == 0xfe02 {
            return read_byte().expect("Interrupted!");
        }

        self.data[addr as usize]
    }

    pub fn write(&mut self, addr: u16, val: u16) {
        self.data[addr as usize] = val;
    }
}
