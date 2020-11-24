#[allow(dead_code)]

use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::process;

fn main() {
    static CYCLES_PER_FRAME: usize = 4_000_000 / 60;


    let condition = ConditionCodes {z:false, s:false, p:false, cy:0, ac:0, pad:0};
    let mut state = State8080 {a:0, b:0, c:0, d:0, e:0, h:0, l:0, sp:0, pc:0, memory:[0;16384],cc:condition, int_enable:0};

    let mut invadersh = File::open("invaders.h").expect("no such file");
    invadersh.read(&mut state.memory[..0x07ff]).expect("error reading into emulated memory");

    let mut invadersg = File::open("invaders.g").expect("no such file");
    invadersg.read(&mut state.memory[0x0800..0x0fff]).expect("error reading into emulated memory");

    let mut invadersf = File::open("invaders.f").expect("no such file");
    invadersf.read(&mut state.memory[0x1000..0x17ff]).expect("error reading into emulated memory");

    let mut invaderse = File::open("invaders.e").expect("no such file");
    invaderse.read(&mut state.memory[0x1800..0x1fff]).expect("error reading into emulated memory");

    //Test if file is read correctly
    println!("{:x?} {:x?} {:x?} {:x?}", state.memory[0], state.memory[1], state.memory[2], state.memory[3]);

    let mut i: usize = 0;
    let mut cycles: usize = 0;
    let mut interrupt_type: bool = false;

    loop {
        while cycles <= CYCLES_PER_FRAME / 2 {
            emulate_instruction(&mut state, &mut cycles);
        }
        //render here
        if state.int_enable == 1{
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

        0x04 => {
            let result = state.b + 1;
            state.cc.z = (result & 0xff) == 0;
            state.cc.s = (result & 0x80) != 0;
            state.cc.p = parity(result & 0xff);
            state.b = result & 0xff;
        },

        //DCR B
        0x05 => {
            let result = state.b -1;
            state.cc.z = (result & 0xff) == 0;
            state.cc.s = (result & 0x80) != 0;
            state.cc.p = parity(result & 0xff);
            state.b = result & 0xff;
        },

        // JMP adress
        0xc3 => state.pc= (u16::from(state.memory[pc + 2]) << 8) | u16::from(state.memory[pc + 1]),
        _ => unimplemented_instruction(),
    }
}

fn parity(x: u8) -> bool {
    //TODO
    return true;
}

fn unimplemented_instruction() {
    println!("Unimplemented Instruction!");
    process::exit(0x0);
}

fn generate_interrupt(state: &mut State8080, interrupt_type: &mut bool) {

}

struct ConditionCodes {
    z: bool,
    s: bool,
    p: bool,
    cy: u8,
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
    int_enable: u8,
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