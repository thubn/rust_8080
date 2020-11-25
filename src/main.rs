#[allow(dead_code)]

use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::process;

fn main() {
    static CYCLES_PER_FRAME: usize = 4_000_000 / 60;


    let condition = ConditionCodes {z:false, s:false, p:false, cy:false, ac:0, pad:0};
    let mut state = State8080 {a:0, b:0, c:0, d:0, e:0, h:0, l:0, sp:0, pc:0, memory:[0;16384],cc:condition, int_enable:false};

    let mut invadersh = File::open("invaders.h").expect("no such file");
    invadersh.read(&mut state.memory[..0x07ff]).expect("error reading into emulated memory");

    let mut invadersg = File::open("invaders.g").expect("no such file");
    invadersg.read(&mut state.memory[0x0800..0x0fff]).expect("error reading into emulated memory");

    let mut invadersf = File::open("invaders.f").expect("no such file");
    invadersf.read(&mut state.memory[0x1000..0x17ff]).expect("error reading into emulated memory");

    let mut invaderse = File::open("invaders.e").expect("no such file");
    invaderse.read(&mut state.memory[0x1800..0x1fff]).expect("error reading into emulated memory");

    //Test if file is read correctly
    let test: u16 = 0xff00;
    let u8test: u8 = test.to_be_bytes()[1];
    println!("{:x?}", u8test);
    println!("{:x?} {:x?} {:x?} {:x?}", state.memory[0], state.memory[1], state.memory[2], state.memory[3]);

    let mut i: usize = 0;
    let mut cycles: usize = 0;
    let mut interrupt_type: bool = false;

    loop {
        while cycles <= CYCLES_PER_FRAME / 2 {
            emulate_instruction(&mut state, &mut cycles);
        }
        //render here
        if state.int_enable {
            generate_interrupt(&mut state, &mut interrupt_type);
        }
        cycles = 0;

        i+=1;
        if i > 1 { break; }
    }
}

fn emulate_instruction(state: &mut State8080, cycles: &mut usize) {
    let opcode: u8 = state.memory[usize::from(state.pc)];
    *cycles += usize::from(CYCLES[usize::from(opcode)]);
    println!("Opcode: {:x?}", opcode);
    let pc: usize = usize::from(state.pc);
    state.pc += 1;

    match opcode {
        // NOP
        0x00 => (),

        // LXI B, word
        0x01 => {
            state.c = state.memory[pc+1];
            state.b = state.memory[pc+2];
            state.pc = state.pc + 2;
        },

        0x02 => unimplemented_instruction(),

        // INX B
        0x03 => {
            state.c += 1;
            if state.c == 0 {state.b += 1;}
        },

        // INR B
        0x04 => {
            let result = state.b + 1;
            state.cc.z = (result & 0xff) == 0;
            state.cc.s = (result & 0x80) != 0;
            state.cc.p = parity(result & 0xff);
            state.b = result & 0xff;
        },

        // DCR B
        0x05 => {
            let result = state.b -1;
            state.cc.z = (result & 0xff) == 0;
            state.cc.s = (result & 0x80) != 0;
            state.cc.p = parity(result & 0xff);
            state.b = result & 0xff;
        },

        // MVI B
        0x06 => {
            state.b = state.memory[pc + 1];
            state.pc += 1;
        },

        // RLC
        0x07 => {
            state.a = (state.a >> 7) & (state.a << 1);
        },

        // DAD B
        0x09 => {
            let hl: u32 = (u32::from(state.h) << 8 | u32::from(state.l)) + (u32::from(state.b) << 8 | u32::from(state.c));
            state.h = hl.to_be_bytes()[0];
            state.l = hl.to_be_bytes()[1];
            state.cc.cy = (hl & 0xffff0000) > 0;
        },

        // LDAX B
        0x0a => {
            let offset: usize = (usize::from(state.b) << 8) | usize::from(state.c);
            state.a = state.memory[offset];
        },

        // DCR C
        0x0d => {
            state.c -= 1;
            state.cc.z = state.c == 0;
            state.cc.s = (state.c & 0x80) != 0;
            state.cc.p = parity(state.c);
        },

        // MVI C
        0x0e => {
            state.c = state.memory[pc + 1];
            state.pc +=1;
        },

        // RRC
        0x0f => {
            state.cc.cy = (state.a & 0x1) == 1;
            state.a = ((state.a & 1) << 7) | (state.a >> 1);
        },

        // LXI D,word
        0x11 => {
            state.e = state.memory[pc + 1];
            state.d = state.memory[pc + 2];
            state.pc += 2;
        },

        // INX D
        0x13 => {
            state.e += 1;
            if state.e == 0 { state.d += 1; }
        },

        // MVI D,D8
        0x16 => {
            state.d = state.memory[pc + 1];
            state.pc += 1;
        },

        // DAD D
        0x19 => {
            let hl: u32 = (u32::from(state.h) << 8 | u32::from(state.l)) + (u32::from(state.d) << 8 | u32::from(state.e));
            state.h = hl.to_be_bytes()[0];
            state.l = hl.to_be_bytes()[1];
            state.cc.cy = (hl & 0xffff0000) > 0;
        },

        // LDAX D
        0x1a => {
            let offset: usize = (usize::from(state.d) << 8) | usize::from(state.e);
            state.a = state.memory[offset];
        },

        // RAR
        0x1f => {
            let x = state.a;
            let mut b: u8;
            if state.cc.cy {
                b = 1;
            }else{
                b = 0;
            }
            state.a = (b << 7) | (x >> 1);
            state.cc.cy = 1 == (x & 0x1);
        },

        // LXI H,word
        0x21 => {
            state.l = state.memory[pc + 1];
            state.h = state.memory[pc + 2];
            state.pc += 2;
        },

        // INX H
        0x23 => {
            state.l += 1;
            if state.l == 0 { state.h += 1; }
        },

        // MVI H
        0x26 => {
            state.h = state.memory[pc + 1];
            state.pc +=1;
        },

        // DAD H
        0x29 => {
            let hl: u32 = (u32::from(state.h) << 8 | u32::from(state.l)) * 2;
            state.h = hl.to_be_bytes()[0];
            state.l = hl.to_be_bytes()[1];
            state.cc.cy = (hl & 0xffff0000) > 0;
        },

        // LHLD adr
        0x2a => {
            let offset: usize = (usize::from(state.memory[pc + 2]) << 8) | usize::from(state.memory[pc + 1]);
            state.l = state.memory[offset];
            state.h = state.memory[offset + 1];
            state.pc += 2;
        },

        // DCX H
        0x2b => {
            state.l -= 1;
            if state.l == 0xff { state.h -= 1; }
        },

        // INR L
        0x2c => {
            state.l += 1;
            state.cc.z = state.l == 0;
            state.cc.s = (state.l & 0x80) != 0;
            state.cc.p = parity(state.l);
        },

        // MVI L,D8
        0x2e => {
            state.l = state.memory[pc + 1];
            state.pc += 1;
        }

        // CMA (not)
        0x2f => {
            state.a = !state.a;
        }

        // LXI SP,word
        0x31 => {
            state.sp = (u16::from(state.memory[pc + 2]) << 8) | u16::from(state.memory[pc + 1]);
            state.pc += 2;
        }

        // JMP adress
        0xc3 => state.pc= (u16::from(state.memory[pc + 2]) << 8) | u16::from(state.memory[pc + 1]),
        _ => unimplemented_instruction(),
    }
}

fn parity(x: u8) -> bool {
    let mut one_bits: u8 = 0;
    for i in 0..8 {
        one_bits += (x >> i) & 0x1;
    }
    return (one_bits & 0x1) != 0;
}

fn unimplemented_instruction() {
    println!("Unimplemented Instruction!");
    process::exit(0x0);
}

fn generate_interrupt(state: &mut State8080, interrupt_type: &mut bool) {
    state.memory[usize::from(state.sp) - 1] = state.pc.to_be_bytes()[0];
    state.memory[usize::from(state.sp) - 2] = state.pc.to_be_bytes()[1];
    state.sp -= 2;
    state.pc = 8*(*interrupt_type as u16);
    state.int_enable = false;
}

struct ConditionCodes {
    z: bool,
    s: bool,
    p: bool,
    cy: bool,
    ac: u8,
    pad: u8,
}

struct State8080 {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    sp: u16,
    pc: u16,
    memory: [u8;16384],
    cc: ConditionCodes,
    int_enable: bool,
}

const CYCLES: [u8; 256] = [
    4, 10, 7, 5, 5, 5, 7, 4, 4, 10, 7, 5, 5, 5, 7, 4,
    4, 10, 7, 5, 5, 5, 7, 4, 4, 10, 7, 5, 5, 5, 7, 4,
    4, 10, 16, 5, 5, 5, 7, 4, 4, 10, 16, 5, 5, 5, 7, 4,
    4, 10, 13, 5, 10, 10, 10, 4, 4, 10, 13, 5, 5, 5, 7, 4,

    5, 5, 5, 5, 5, 5, 7, 5, 5, 5, 5, 5, 5, 5, 7, 5,
    5, 5, 5, 5, 5, 5, 7, 5, 5, 5, 5, 5, 5, 5, 7, 5,
    5, 5, 5, 5, 5, 5, 7, 5, 5, 5, 5, 5, 5, 5, 7, 5,
    7, 7, 7, 7, 7, 7, 7, 7, 5, 5, 5, 5, 5, 5, 7, 5,

    4, 4, 4, 4, 4, 4, 7, 4, 4, 4, 4, 4, 4, 4, 7, 4,
    4, 4, 4, 4, 4, 4, 7, 4, 4, 4, 4, 4, 4, 4, 7, 4,
    4, 4, 4, 4, 4, 4, 7, 4, 4, 4, 4, 4, 4, 4, 7, 4,
    4, 4, 4, 4, 4, 4, 7, 4, 4, 4, 4, 4, 4, 4, 7, 4,

    11, 10, 10, 10, 17, 11, 7, 11, 11, 10, 10, 10, 10, 17, 7, 11,
    11, 10, 10, 10, 17, 11, 7, 11, 11, 10, 10, 10, 10, 17, 7, 11,
    11, 10, 10, 18, 17, 11, 7, 11, 11, 5, 10, 5, 17, 17, 7, 11,
    11, 10, 10, 4, 17, 11, 7, 11, 11, 5, 10, 4, 17, 17, 7, 11
];