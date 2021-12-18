#[derive(Debug)]
pub struct Instructioninfo {
    pub instr_n: usize,
    pub opcode: u8,
    pub pc: u16,
    pub sp: u16,
    pub int_enable: bool,
}