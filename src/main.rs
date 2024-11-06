use std::{fs::File, io::Read};

pub mod opcodes;

struct Cpu {
    pub registers: [u16; 8],
    pub memory: [u16; 0x10000],
    pub pc: u16,
    pub psr: u16,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            registers: [0; 8],
            memory: [0; 0x10000],
            pc: 0x3000,
            psr: 0,
        }
    }

    pub fn step(&mut self) {
        // read instruction
        let instr = self.memory[self.pc as usize];
        self.pc += 1;

        // do stuff
        let opcode = instr >> 12;

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
                let flag = (instr >> 11) & 0b1;

                if flag == 0 {
                    self.pc = (instr >> 6) & 0b111;
                } else {
                    self.pc = self.pc.wrapping_add(self.sext(instr & 0b11111111111, 11));
                }
            }
            opcodes::LD => {
                let dr = (instr >> 9) & 0b111;

                let val = self.memory[(self.pc + self.sext(instr & 0b111111111, 9)) as usize];

                self.registers[dr as usize] = val;
                self.setcc(val);
            }
            opcodes::LDI => {
                let dr = (instr >> 9) & 0b111;

                let val = self.memory
                    [self.memory[(self.pc + self.sext(instr & 0b111111111, 9)) as usize] as usize];

                self.registers[dr as usize] = val;
                self.setcc(val);
            }
            opcodes::LDR => {
                let dr = (instr >> 9) & 0b111;
                let baser = (instr >> 6) & 0b111;

                let val = self.memory
                    [(self.registers[baser as usize] + self.sext(instr & 0b111111, 6)) as usize];

                self.registers[dr as usize] = val;
                self.setcc(val);
            }
            opcodes::LEA => {
                let dr = (instr >> 9) & 0b111;

                let val = self.pc + self.sext(instr & 0b111111111, 9);

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
                self.memory[(self.pc + self.sext(instr & 0b111111111, 9)) as usize] =
                    self.registers[((instr >> 9) & 0b111) as usize];
            }
            opcodes::STI => {
                self.memory[self.memory[(self.pc + self.sext(instr & 0b111111111, 9)) as usize]
                    as usize] = self.registers[((instr >> 9) & 0b111) as usize];
            }
            opcodes::STR => {
                let offset6 = instr & 0b111111;
                let baser = (instr >> 6) & 0b111;

                self.memory[(self.registers[baser as usize] + self.sext(offset6, 6)) as usize] =
                    (instr >> 9) & 0b111;
            }
            opcodes::TRAP => {
                self.registers[7] = self.pc;

                let trapvect = instr & 0b11111111;

                match trapvect {
                    // getc
                    0x20 => {}
                    // out
                    0x21 => {
                        print!("{}", (self.registers[0] as u8) as char);
                    }
                    // puts
                    0x22 => {
                        let mut addr = self.registers[0] as usize;

                        let mut c = 1;
                        while c != 0 {
                            c = self.memory[addr];

                            print!("{}", (c as u8) as char);
                            addr += 1;
                        }
                    }
                    // in
                    0x23 => {}
                    // putsp
                    0x24 => {
                        let mut addr = self.registers[0] as usize;

                        loop {
                            let c = self.memory[addr];

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

    fn setcc(&mut self, val: u16) {
        self.psr &= 0b1111111111111000;

        let n = (val >> 15) & 1;
        let z = (val == 0) as u16;
        let p = !n & !z;

        self.psr |= (n << 2) | (z << 1) | p;
    }
}

fn main() {
    let mut cpu = Cpu::new();

    let mut file = File::open("helloworld").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();

    let start_addr = &buffer[..2];
    cpu.pc = ((start_addr[0] as u16) << 8) | start_addr[1] as u16;

    let code = &buffer[2..];
    for (i, chunk) in code.chunks(2).enumerate() {
        let value = ((chunk[0] as u16) << 8) | chunk[1] as u16;
        cpu.memory[cpu.pc as usize + i] = value;
    }

    loop {
        cpu.step();
    }
}