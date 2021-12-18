
pub use crate::cpu::state8080::State8080;
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
    set_arithmetic_flags(cc, &u16::from(*register));
    /*cc.z = zero(register);
    cc.s = sign(register);
    cc.p = parity(register);
    cc.ac = (*register & 0xf) ==0;*/
}

pub fn dcr(register: &mut u8, cc: &mut ConditionCodes) {
    *register = register.wrapping_sub(1);
    set_arithmetic_flags(cc, &u16::from(*register));
    /*cc.z = zero(register);
    cc.s = sign(register);
    cc.p = parity(register);
    cc.ac = !((*register & 0xf) == 0);*/
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
    let result : u16 = u16::from(*tregister) + u16::from(*sregister) + cc.cy as u16;
    *tregister = result as u8;
    set_arithmetic_flags(cc, &result)
    /*cc.cy = result > 0xff;
    cc.ac = carry(4, *tregister, *sregister, cc.cy);
    cc.z = zero(tregister);
    cc.s = sign(tregister);
    cc.p = parity(tregister);*/
}

pub fn ana(tregister: &mut u8, sregister: &u8, cc: &mut ConditionCodes) {
    let result = *tregister & *sregister;
    //cc.ac = ((*tregister | *sregister) & 0x08) != 0;
    *tregister = result;
    set_arithmetic_flags(cc, &u16::from(result))
    /*cc.z = zero(tregister);
    cc.s = sign(tregister);
    cc.p = parity(tregister);
    cc.cy = false;*/
}

pub fn xra(tregister: &mut u8, sregister: &u8, cc: &mut ConditionCodes) {
    *tregister = *tregister ^ *sregister;
    set_arithmetic_flags(cc, &u16::from(*tregister));
    /*cc.z = zero(tregister);
    cc.s = sign(tregister);
    cc.p = parity(tregister);
    cc.cy = false;
    cc.ac = false;*/
}

pub fn ora(tregister: &mut u8, sregister: &u8, cc: &mut ConditionCodes) {
    *tregister = *tregister | *sregister;
    set_arithmetic_flags(cc, &u16::from(*tregister));
    /*cc.z = zero(tregister);
    cc.s = sign(tregister);
    cc.p = parity(tregister);
    cc.cy = false;
    cc.ac = false;*/
}

pub fn cmp(tregister: &mut u8, sregister: &u8, cc: &mut ConditionCodes) {
    let result = u16::from(*tregister).wrapping_sub(u16::from(*sregister));
    //cc.cy = result >> 8 != 0;
    set_arithmetic_flags(cc, &result);
    /*let result = tregister.wrapping_sub(*sregister);
    cc.ac = (*tregister ^ result ^ *sregister) & 0x10 == 0;
    cc.z = zero(&result);
    cc.s = sign(&result);
    cc.p = parity(&result);*/
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

fn carry(bit_no: u8, a: u8, b: u8, cy: bool) -> bool {
    let result: u16 = a as u16 + b as u16 + cy as u16;
    let carry: u16 = result ^ a as u16 ^ b as u16;
    return (carry & (1 << bit_no as u16)) != 0;
}



pub fn set_arithmetic_flags(cc: &mut ConditionCodes, val: &u16){
    cc.z = (*val & 0xff) == 0;
    cc.s = 0x80 == (*val & 0x80);
    cc.p = (*val & 0xff).count_ones() % 2 == 0;
    cc.cy = *val >> 8 != 0;
    cc.ac = cc.cy;
}