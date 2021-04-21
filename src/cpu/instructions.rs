
pub use crate::cpu::State8080;
pub use crate::cpu::ConditionCodes;

//0x00 0x08 0x10 0x18 0x20 0x28 0x30 0x38
pub fn nop() {}

//0x01 0x11 0x21 0x31
pub fn lxi(high: &mut u8, low: &mut u8, pc: &usize, memory: &[u8;16384]) {
    *high = memory[*pc + 2];
    *low = memory[*pc + 1];
}

pub fn inr(register: &mut u8, cc: &mut ConditionCodes) {
    *register = register.wrapping_add(1);
    cc.z = zero(register);
    cc.s = sign(register);
    cc.p = parity(register);
}

pub fn dcr(register: &mut u8, cc: &mut ConditionCodes) {
    *register = register.wrapping_sub(1);
    cc.z = zero(register);
    cc.s = sign(register);
    cc.p = parity(register);
}

pub fn inx(high: &mut u8, low: &mut u8) {
    *low = low.wrapping_add(1);
    if *low == 0 {*high = high.wrapping_add(1);}
}

pub fn dad(h: &mut u8, l: &mut u8, high: &u8, low: &u8, cc: &mut ConditionCodes){
    let hl: u32 = ((u32::from(*h) << 8) | u32::from(*l)) + ((u32::from(*high) << 8) | u32::from(*low));
    *h = hl.to_be_bytes()[2];
    *l = hl.to_be_bytes()[3];
    cc.cy = (hl & 0xffff0000) > 0;
}

pub fn dcx(high: &mut u8, low: &mut u8) {
    *low = low.wrapping_sub(1);
    if *low == 0xff { *high = high.wrapping_sub(1);}
}

pub fn stax(register: &u8, high: &u8, low: &u8, memory: &mut [u8;16384]) {
    let offset: usize = (usize::from(*high) << 8) | usize::from(*low);
    memory[offset] = *register;
}

pub fn ldax(register: &mut u8, high: &u8, low: &u8, memory: &[u8;16384]) {
    let offset: usize = (usize::from(*high) << 8) | usize::from(*low);
    *register = memory[offset];
}

pub fn mvi(register: &mut u8, memory: &[u8;16384], pc: &usize) {
    *register= memory[*pc + 1];
}

pub fn mov(tregister: &mut u8, sregister: &u8) {
    *tregister = *sregister;
}

pub fn add(tregister: &mut u8, sregister: &u8, cc: &mut ConditionCodes) {
    let result : u16 = u16::from(*tregister) + u16::from(*sregister);
    *tregister = result as u8;
    cc.cy = result > 0xff;
    cc.z = zero(tregister);
    cc.s = sign(tregister);
    cc.p = parity(tregister);
}

pub fn ana(tregister: &mut u8, sregister: &u8, cc: &mut ConditionCodes) {
    *tregister = *tregister & *sregister;
    cc.z = zero(tregister);
    cc.s = sign(tregister);
    cc.p = parity(tregister);
    cc.cy = false;
    cc.ac = false;
}

pub fn xra(tregister: &mut u8, sregister: &u8, cc: &mut ConditionCodes) {
    *tregister = *tregister ^ *sregister;
    cc.z = zero(tregister);
    cc.s = sign(tregister);
    cc.p = parity(tregister);
    cc.cy = false;
    cc.ac = false;
}

pub fn ora(tregister: &mut u8, sregister: &u8, cc: &mut ConditionCodes) {
    *tregister = *tregister | *sregister;
    cc.z = zero(tregister);
    cc.s = sign(tregister);
    cc.p = parity(tregister);
    cc.cy = false;
    cc.ac = false;
}

fn parity(x: &u8) -> bool {
    let mut one_bits: u8 = 0;
    for i in 0..8 {
        one_bits += (x >> i) & 0x1;
    }
    return (one_bits & 0x1) != 0;
}

fn zero(x: &u8) -> bool {
    return *x == 0;
}

fn sign(x: &u8) -> bool {
    return (*x & 0x80) != 0;
}