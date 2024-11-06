pub const ADD: u16 = 0b0001;
pub const AND: u16 = 0b0101;
pub const BR: u16 = 0b0000;
pub const JMP: u16 = 0b1100;
pub const JSR: u16 = 0b0100;
pub const JSRR: u16 = 0b0100;
pub const LD: u16 = 0b0010;
pub const LDI: u16 = 0b1010;
pub const LDR: u16 = 0b0110;
pub const LEA: u16 = 0b1110;
pub const NOT: u16 = 0b1001;
pub const RET: u16 = 0b1100;
pub const RTI: u16 = 0b1000;
pub const ST: u16 = 0b0011;
pub const STI: u16 = 0b1011;
pub const STR: u16 = 0b0111;
pub const TRAP: u16 = 0b1111;

pub fn print_opcode(opcode: u16) -> String {
    match opcode {
        ADD => "ADD",
        AND => "AND",
        BR => "BR",
        JMP => "JMP",
        JSR => "JSR",
        //JSRR => "JSRR",
        LD => "LD",
        LDI => "LDI",
        LDR => "LDR",
        LEA => "LEA",
        NOT => "NOT",
        //RET => "RET",
        RTI => "RTI",
        ST => "ST",
        STI => "STI",
        STR => "STR",
        TRAP => "TRAP",
        _ => "Unknown opcode",
    }
    .to_string()
}
