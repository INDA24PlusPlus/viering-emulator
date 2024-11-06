use std::{
    fs::File,
    io::{self, Read, Write},
};

use memory::Memory;
use termion::raw::IntoRawMode;

pub mod memory;
pub mod opcodes;

struct Cpu {
    pub registers: [u16; 8],
    pub memory: Memory,
    pub pc: u16,
    pub psr: u16,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            registers: [0; 8],
            memory: Memory::new(),
            pc: 0x3000,
            psr: 0,
        }
    }

    pub fn step(&mut self, debug_mode: bool) {
        // read instruction
        let instr = self.memory.read(self.pc);
        self.pc = self.pc.wrapping_add(1);

        // do stuff
        let opcode = instr >> 12;

        if debug_mode {
            println!(
                "{:#X}: opcode: {}, psr: {:#X}, registers: {:X?}",
                self.pc - 1,
                opcodes::print_opcode(opcode),
                self.psr,
                self.registers
            );
        }

        match opcode {
            opcodes::ADD => {
                let flag = (instr >> 5) & 0b1;
                let dr = (instr >> 9) & 0b111;
                let sr1 = (instr >> 6) & 0b111;

                let val = if flag == 0 {
                    let sr2 = instr & 0b111;
                    self.registers[sr1 as usize].wrapping_add(self.registers[sr2 as usize])
                } else {
                    let imm5 = instr & 0b11111;
                    self.registers[sr1 as usize].wrapping_add(self.sext(imm5, 5))
                };

                self.registers[dr as usize] = val;
                self.setcc(val);
            }
            opcodes::AND => {
                let flag = (instr >> 5) & 0b1;
                let dr = (instr >> 9) & 0b111;
                let sr1 = (instr >> 6) & 0b111;

                let val = if flag == 0 {
                    let sr2 = instr & 0b111;
                    self.registers[sr1 as usize] & self.registers[sr2 as usize]
                } else {
                    let imm5 = instr & 0b11111;
                    self.registers[sr1 as usize] & self.sext(imm5, 5)
                };

                self.registers[dr as usize] = val;
                self.setcc(val);
            }
            opcodes::BR => {
                let condition = ((instr >> 9) & 0b111) & (self.psr & 0b111);
                if condition != 0 {
                    self.pc = self.pc.wrapping_add(self.sext(instr & 0b111111111, 9));
                }
            }
            // JMP and RET
            opcodes::JMP => {
                let addr = (instr >> 6) & 0b111;
                self.pc = self.registers[addr as usize];
            }
            // JSR and JSRR
            opcodes::JSR => {
                self.registers[7] = self.pc;

                let flag = (instr >> 11) & 0b1;

                if flag == 0 {
                    self.pc = self.registers[((instr >> 6) & 0b111) as usize];
                } else {
                    self.pc = self.pc.wrapping_add(self.sext(instr & 0b11111111111, 11));
                }
            }
            opcodes::LD => {
                let dr = (instr >> 9) & 0b111;

                let val = self
                    .memory
                    .read(self.pc.wrapping_add(self.sext(instr & 0b111111111, 9)));

                self.registers[dr as usize] = val;
                self.setcc(val);
            }
            opcodes::LDI => {
                let dr = (instr >> 9) & 0b111;

                let val = self.memory.read(
                    self.memory
                        .read(self.pc.wrapping_add(self.sext(instr & 0b111111111, 9))),
                );

                self.registers[dr as usize] = val;
                self.setcc(val);
            }
            opcodes::LDR => {
                // FUCK ME THIS IS WRONG PROBABLY BUT IT DOESNT LOOK WRONG
                let dr = (instr >> 9) & 0b111;
                let baser = (instr >> 6) & 0b111;

                let val = self.memory.read(
                    self.registers[baser as usize].wrapping_add(self.sext(instr & 0b111111, 6)),
                );

                self.registers[dr as usize] = val;
                self.setcc(val);
            }
            opcodes::LEA => {
                let dr = (instr >> 9) & 0b111;

                let val = self.pc.wrapping_add(self.sext(instr & 0b111111111, 9));

                self.registers[dr as usize] = val;
                self.setcc(val);
            }
            opcodes::NOT => {
                let dr = (instr >> 9) & 0b111;

                let val = !self.registers[((instr >> 6) & 0b111) as usize];

                self.registers[dr as usize] = val;
                self.setcc(val);
            }
            opcodes::RTI => {
                panic!("Not supported!");
            }
            opcodes::ST => {
                self.memory.write(
                    self.pc.wrapping_add(self.sext(instr & 0b111111111, 9)),
                    self.registers[((instr >> 9) & 0b111) as usize],
                );
            }
            opcodes::STI => {
                self.memory.write(
                    self.memory
                        .read(self.pc.wrapping_add(self.sext(instr & 0b111111111, 9))),
                    self.registers[((instr >> 9) & 0b111) as usize],
                );
            }
            opcodes::STR => {
                let offset6 = instr & 0b111111;
                let baser = (instr >> 6) & 0b111;

                self.memory.write(
                    self.registers[baser as usize].wrapping_add(self.sext(offset6, 6)),
                    self.registers[((instr >> 9) & 0b111) as usize],
                );
            }
            opcodes::TRAP => {
                self.registers[7] = self.pc;

                let trapvect = instr & 0b11111111;

                match trapvect {
                    // getc
                    0x20 => match read_byte() {
                        Some(byte) => {
                            self.registers[0] = byte;
                        }
                        None => {
                            println!();
                            std::process::exit(0)
                        }
                    },
                    // out
                    0x21 => {
                        print!("{}", (self.registers[0] as u8) as char);
                    }
                    // puts
                    0x22 => {
                        let mut addr = self.registers[0] as usize;

                        let mut c = 1;
                        while c != 0 {
                            c = self.memory.read(addr as u16);

                            print!("{}", (c as u8) as char);
                            addr += 1;
                        }
                    }
                    // in
                    0x23 => {
                        print!("\n > ");
                        io::stdout().flush().unwrap();

                        match read_byte() {
                            Some(byte) => {
                                self.registers[0] = byte;
                                print!("{}", (byte as u8) as char);
                                io::stdout().flush().unwrap();
                            }
                            None => {
                                println!();
                                std::process::exit(0)
                            }
                        }
                    }
                    // putsp
                    0x24 => {
                        let mut addr = self.registers[0] as usize;

                        loop {
                            let c = self.memory.read(addr as u16);

                            let c1 = c & 0b11111111;
                            let c2 = c >> 8;

                            if c1 != 0 {
                                print!("{}", (c1 as u8) as char);
                            }
                            if c2 != 0 {
                                print!("{}", (c2 as u8) as char);
                            }

                            if c1 == c2 && c1 == 0 {
                                break;
                            }

                            addr += 1;
                        }
                    }
                    // halt
                    0x25 => {
                        println!();
                        std::process::exit(0)
                    }
                    _ => panic!("Invalid trap call!"),
                };
            }
            _ => println!("Invalid opcode!"),
        }
    }

    fn sext(&self, value: u16, bits: u8) -> u16 {
        let shift = 16 - bits;
        (((value << shift) as i16) >> shift) as u16
    }

    fn setcc(&mut self, value: u16) {
        let n = (value >> 15) == 1 && value != 0;
        let z = value == 0;
        let p = (value >> 15) == 0 && value != 0;

        self.psr &= 0b1111111111111000;
        self.psr |= ((n as u16) << 2) | ((z as u16) << 1) | (p as u16);
    }
}

fn read_byte() -> Option<u16> {
    let _i_am_very_important_hoorah = io::stdout().into_raw_mode().unwrap();

    let mut buffer = [0u8];

    io::stdin().read_exact(&mut buffer).unwrap();

    if buffer[0] == 3 {
        return None;
    }

    if buffer[0] == b'\r' {
        return Some(b'\n' as u16);
    }

    Some(buffer[0] as u16)
}

fn main() {
    let mut cpu = Cpu::new();

    let mut file = File::open("examples/2048.obj").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();

    let start_addr = &buffer[..2];
    cpu.pc = ((start_addr[0] as u16) << 8) | start_addr[1] as u16;

    let code = &buffer[2..];
    for (i, chunk) in code.chunks(2).enumerate() {
        let value = ((chunk[0] as u16) << 8) | chunk[1] as u16;
        cpu.memory.write(cpu.pc + i as u16, value);
    }

    loop {
        cpu.step(false);
    }
}
