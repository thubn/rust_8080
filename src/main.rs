#[allow(dead_code)]
//mod render;
mod cpu;

pub use crate::cpu::debugging::Instructioninfo;
pub use crate::cpu::instructions;
pub use crate::cpu::state8080::ConditionCodes;
pub use crate::cpu::state8080::State8080;

use minifb::{Window, /*ScaleMode,*/ WindowOptions};
//use std::io;
use std::fs::File;
use std::io::Read;
use std::process;
use std::process::exit;
use std::{thread, time};

const SCREEN_WIDTH: usize = 224;
const SCREEN_HEIGHT: usize = 256;
const NUM_PIXELS: usize = SCREEN_HEIGHT * SCREEN_WIDTH;

fn main() {
    //2MHz/1Mhz with two interrupts for each frame on 60Hz screen
    static CYCLES_PER_FRAME: isize = 1_000_000 / 60;

    let condition: ConditionCodes = ConditionCodes {
        z: false,
        s: false,
        p: false,
        cy: false,
        ac: false,
        pad: 0,
    };
    let mut state: State8080 = State8080 {
        a: 0,
        b: 0,
        c: 0,
        d: 0,
        e: 0,
        h: 0,
        l: 0,
        sp: 0xF000,
        pc: 0,
        memory: [0; 16384],
        cc: condition,
        int_enable: false,
    };
    let mut special: Special = Special {
        shift_offset: 0,
        shift0: 0,
        shift1: 0,
    };

    let mut invadersh: File = File::open("invaders.h").expect("no such file");
    invadersh
        .read(&mut state.memory[..=0x07ff])
        .expect("error reading into emulated memory");

    let mut invadersg = File::open("invaders.g").expect("no such file");
    invadersg
        .read(&mut state.memory[0x0800..=0x0fff])
        .expect("error reading into emulated memory");

    let mut invadersf = File::open("invaders.f").expect("no such file");
    invadersf
        .read(&mut state.memory[0x1000..=0x17ff])
        .expect("error reading into emulated memory");

    let mut invaderse = File::open("invaders.e").expect("no such file");
    invaderse
        .read(&mut state.memory[0x1800..=0x1fff])
        .expect("error reading into emulated memory");

    let mut window = Window::new(
        "8080",
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    window.limit_update_rate(Some(std::time::Duration::from_millis(4)));

    let mut i: usize = 0;
    let mut cycles: isize = 0;
    let mut interrupt_type: bool = false;
    let mut total_instructions: usize = 0;
    //let mut instr_history: Vec<Instructioninfo> = Vec::new();
    let mut instr_counter = [[0usize; 2]; 256];

    //main emulation loop
    loop {
        while cycles <= CYCLES_PER_FRAME / 2 {
            /*instr_history.push(Instructioninfo {
                instr_n: total_instructions,
                opcode: state.memory[usize::from(state.pc)],
                pc: state.pc,
                sp: state.sp,
                int_enable: state.int_enable,
            });*/

            instr_counter[usize::from(state.memory[usize::from(state.pc)])][0] += 1;
            instr_counter[usize::from(state.memory[usize::from(state.pc)])][1] = total_instructions;

            if !emulate_instruction(
                &mut state,
                &mut cycles,
                &mut special,
                &mut total_instructions,
            ) {
                /*let mut count: usize = 1000;
                while let Some(top) = instr_history.pop() {
                    println!("{:x?}", top);
                    count -= 1;
                    if count == 0 {
                        break;
                    };
                }
                //println!();
                for n in 0..256 {
                    println!("Instr: {:x?} count: {:?} last: {:?}", n, instr_counter[n][0], instr_counter[n][1])
                }*/
                println!("Emulation aborted due to an error");
                process::exit(0x0);
            }
            total_instructions += 1;
        }

        let mut buffer: Vec<u32> = vec![0; NUM_PIXELS];
        let mut j = 0;
        for row in (0x2400..=0x241f).rev() {
            for b in (0..=7).rev() {
                for col in 0..224 {
                    let offset = row + (col * 0x20);
                    if (state.memory[offset] & (0x1 << b)) != 0x0 {
                        buffer[j] = 0x00ffffff;
                        //print!("0");
                        //process::exit(0);
                    } else {
                        buffer[j] = 0x00000000;
                        //print!("_");
                        //process::exit(0);
                    }
                    j += 1;
                }
                //println!("");
            }
        }

        window
            .update_with_buffer(&buffer, SCREEN_WIDTH, SCREEN_HEIGHT)
            .unwrap_or_else(|e| {
                panic!("{}", e);
            });

        if state.int_enable {
            generate_interrupt(&mut state, &mut interrupt_type);
            interrupt_type = !interrupt_type;
            i += 1;
            //println!("Interrup No: {}", i);
        }
        cycles = cycles - (CYCLES_PER_FRAME / 2);

        //if i > 100 { break; }
        if total_instructions > 50000000 {
            println!("total_instructions ({:?}) > 50000000, exiting...", total_instructions);
            thread::sleep(time::Duration::from_secs(10));
            break;
        }
        thread::sleep(time::Duration::from_millis(8));
    }
    /*for (i,item) in state.memory[0x2400..=0x3fff].iter().enumerate() {
        println!("{:x?} {:x?}", 0x2400+i, item);
    } */
}

fn emulate_instruction(
    state: &mut State8080,
    cycles: &mut isize,
    special: &mut Special,
    total_instructions: &mut usize,
) -> bool {
    let opcode: u8 = state.memory[usize::from(state.pc)];
    *cycles += isize::from(CYCLES[usize::from(opcode)]);
    let pc: usize = usize::from(state.pc);
    //println!("Instruction: {} op: {:x?} pc:{:x?}", total_instructions, opcode, pc);
    //println!("a:{:x?} bc:{:x?}{:x?} de:{:x?}{:x?} hl:{:x?}{:x?} sp:{:x?}", state.a, state.b, state.c, state.d, state.e, state.h, state.l, state.sp);
    //println!("cycles:{}", *cycles);

    //state.pc += 1;
    state.pc += SIZE[usize::from(opcode)] as u16;

    //let memory = state.memory.clone();

    //for debugging
    /*
    if pc == 0x118/* || pc == 0x100 || pc == 0x141 || pc == 0x17a || pc == 0x1a1 || pc == 0x1c0*/ {
        println!("PC is at {:x?}. Press enter to continue", pc);
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).expect("Did not enter a correct string");
    }*/

    //dump_memory(&state);
    match opcode {
        // NOP
        0x00 => instructions::nop(),

        // LXI B, word
        0x01 => {
            instructions::lxi(&mut state.b, &mut state.c, &pc, &state.memory);
        }

        // INX B
        0x03 => {
            instructions::inx(&mut state.b, &mut state.c);
        }

        // INR B
        0x04 => {
            instructions::inr(&mut state.b, &mut state.cc);
        }

        // DCR B
        0x05 => {
            instructions::dcr(&mut state.b, &mut state.cc);
        }

        // MVI B
        0x06 => {
            instructions::mvi(&mut state.b, &mut state.memory, &pc);
        }

        // RLC
        0x07 => {
            state.cc.cy = (state.a >> 7) != 0;
            state.a = (state.a >> 7) | (state.a << 1);
        }

        // DAD B
        0x09 => {
            instructions::dad(
                &mut state.h,
                &mut state.l,
                &state.b,
                &state.c,
                &mut state.cc,
            );
        }

        // LDAX B
        0x0a => {
            instructions::ldax(&mut state.a, &state.b, &state.c, &state.memory);
        }

        // INR C
        0x0c => {
            instructions::inr(&mut state.c, &mut state.cc);
        }

        // DCR C
        0x0d => {
            instructions::dcr(&mut state.c, &mut state.cc);
        }

        // MVI C
        0x0e => {
            instructions::mvi(&mut state.c, &state.memory, &pc);
        }

        // RRC
        0x0f => {
            state.cc.cy = (state.a & 0x1) != 0;
            state.a = (state.a & 1 << 7) | (state.a >> 1);
        }

        // LXI D,word
        0x11 => {
            instructions::lxi(&mut state.d, &mut state.e, &pc, &state.memory);
        }

        // STAX D
        0x12 => {
            instructions::stax(&state.a, &state.d, &state.e, &mut state.memory);
        }

        // INX D
        0x13 => {
            instructions::inx(&mut state.d, &mut state.e);
        }

        // INR D
        0x14 => instructions::inr(&mut state.d, &mut state.cc),

        // DCR D
        0x15 => instructions::dcr(&mut state.d, &mut state.cc),

        // MVI D,D8
        0x16 => {
            instructions::mvi(&mut state.d, &state.memory, &pc);
        }

        // DAD D
        0x19 => {
            instructions::dad(
                &mut state.h,
                &mut state.l,
                &state.d,
                &state.e,
                &mut state.cc,
            );
        }

        // LDAX D
        0x1a => {
            instructions::ldax(&mut state.a, &state.d, &state.e, &state.memory);
        }

        // RAR
        0x1f => {
            let cy: bool = state.cc.cy;
            state.cc.cy = state.a & 1 != 0;
            state.a = (state.a >> 1) | ((cy as u8) << 7);
        }

        // LXI H,word
        0x21 => {
            instructions::lxi(&mut state.h, &mut state.l, &pc, &state.memory);
        }

        //SHLD
        0x22 => {
            let offset: usize =
                (usize::from(state.memory[pc + 2]) << 8) | usize::from(state.memory[pc + 1]);
            state.memory[offset] = state.l;
            state.memory[offset + 1] = state.h;
        }

        // INX H
        0x23 => {
            instructions::inx(&mut state.h, &mut state.l);
        }

        // MVI H
        0x26 => {
            instructions::mvi(&mut state.h, &state.memory, &pc);
        }

        // DAD H
        0x29 => {
            let h = state.h;
            let l = state.l;
            instructions::dad(&mut state.h, &mut state.l, &h, &l, &mut state.cc);
        }

        // LHLD adr
        0x2a => {
            let offset: usize =
                (usize::from(state.memory[pc + 2]) << 8) | usize::from(state.memory[pc + 1]);
            state.l = state.memory[offset];
            state.h = state.memory[offset + 1];
        }

        // DCX H
        0x2b => {
            instructions::dcx(&mut state.h, &mut state.l);
        }

        // INR L
        0x2c => {
            instructions::inr(&mut state.l, &mut state.cc);
        }

        // MVI L,D8
        0x2e => {
            instructions::mvi(&mut state.l, &state.memory, &pc);
        }

        // CMA (not)
        0x2f => {
            state.a = !state.a;
        }

        // LXI SP,word
        0x31 => {
            //instructions::lxi(&mut state.sp.to_be_bytes()[0], &mut state.sp.to_be_bytes()[0], &pc, &state.memory);
            state.sp = (u16::from(state.memory[pc + 2]) << 8) | u16::from(state.memory[pc + 1]);
        }

        // STA adr
        0x32 => {
            let offset =
                (usize::from(state.memory[pc + 2]) << 8) | usize::from(state.memory[pc + 1]);
            state.memory[offset] = state.a;
        }

        // INR M
        0x34 => {
            let offset = (usize::from(state.h) << 8) | usize::from(state.l);
            instructions::inr(&mut state.memory[offset], &mut state.cc);
        }

        // DCR M
        0x035 => {
            let offset = (usize::from(state.h) << 8) | usize::from(state.l);
            instructions::dcr(&mut state.memory[offset], &mut state.cc);
            /*state.memory[offset] = state.memory[offset].wrapping_sub(1);
            state.cc.z = state.memory[offset] == 0;
            state.cc.s = (state.memory[offset] & 0x80) != 0;
            state.cc.p = parity(state.memory[offset]);*/
        }

        // MVI M,D8
        0x36 => {
            let offset = (usize::from(state.h) << 8) | usize::from(state.l);
            state.memory[offset] = state.memory[pc + 1];
        }

        // STC
        0x37 => {
            state.cc.cy = true;
        }

        // LDA adr
        0x3a => {
            let offset = usize::from(state.memory[pc + 2]) << 8 | usize::from(state.memory[pc + 1]);
            state.a = state.memory[offset];
        }

        // INR A
        0x3c => instructions::inr(&mut state.a, &mut state.cc),

        // DCR A
        0x3d => {
            instructions::dcr(&mut state.a, &mut state.cc);
        }

        // MVI A,D8
        0x3e => {
            instructions::mvi(&mut state.a, &state.memory, &pc);
        }

        // CMC
        0x3f => {
            state.cc.cy = !state.cc.cy;
        }

        // MOV B,B
        0x40 => (),

        // MOV B,C
        0x41 => state.b = state.c,

        // MOV B,D
        0x42 => state.b = state.d,

        // MOV B,E
        0x43 => state.b = state.e,

        // MOV B,H
        0x44 => state.b = state.h,

        // MOV B,L
        0x45 => state.b = state.l,

        // MOV B,M
        0x46 => {
            let offset = (usize::from(state.h) << 8) | usize::from(state.l);
            state.b = state.memory[offset];
        }

        // MOV B,A
        0x47 => state.b = state.a,

        // MOV C,M
        0x4e => {
            let offset = (usize::from(state.h) << 8) | usize::from(state.l);
            state.c = state.memory[offset];
        }

        // MOV C,A
        0x4f => {
            state.c = state.a;
        }

        // MOV D,M
        0x56 => {
            let offset = (usize::from(state.h) << 8) | usize::from(state.l);
            state.d = state.memory[offset];
        }

        // MOV D,A
        0x57 => {
            state.d = state.a;
        }

        // MOV E,M
        0x5e => {
            let offset = (usize::from(state.h) << 8) | usize::from(state.l);
            state.e = state.memory[offset];
        }

        // MOV E,A
        0x5f => {
            state.e = state.a;
        }

        // MOV H,C
        0x61 => state.h = state.c,

        // MOV H,L
        0x65 => state.h = state.l,

        // MOV H,M
        0x66 => {
            let offset = (usize::from(state.h) << 8) | usize::from(state.l);
            state.h = state.memory[offset];
        }

        // MOV H,A
        0x67 => {
            state.h = state.a;
        }

        // MOV L,B
        0x68 => state.l = state.b,

        // MOV L,C
        0x69 => state.l = state.c,

        // MOV L,A
        0x6f => {
            state.l = state.a;
        }

        // MOV M,B
        0x70 => {
            let offset = (usize::from(state.h) << 8) | usize::from(state.l);
            state.memory[offset] = state.b;
        }

        // MOV M,C
        0x71 => {
            let offset = (usize::from(state.h) << 8) | usize::from(state.l);
            state.memory[offset] = state.c;
        }

        // MOV M,A
        0x77 => {
            let offset = (usize::from(state.h) << 8) | usize::from(state.l);
            if offset >= state.memory.len() {
                println!("offset out of bounds({:x?})", offset);
                exit(1);
            }
            state.memory[offset] = state.a;
        }

        // MOV A,B
        0x78 => {
            state.a = state.b;
        }

        // MOV A,C
        0x79 => {
            state.a = state.c;
        }

        // MOV A,D
        0x7a => {
            state.a = state.d;
        }

        // MOV A,E
        0x7b => {
            state.a = state.e;
        }

        // MOV A,H
        0x7c => {
            state.a = state.h;
        }

        // MOV A,L
        0x7d => {
            state.a = state.l;
        }

        // MOV A,M
        0x7e => {
            let offset = (usize::from(state.h) << 8) | usize::from(state.l);
            state.a = state.memory[offset];
        }

        // ADD B
        0x80 => {
            instructions::add(&mut state.a, &state.b, &mut state.cc);
        }

        // ADD C
        0x81 => {
            instructions::add(&mut state.a, &state.c, &mut state.cc);
        }

        // ADD D
        0x82 => {
            instructions::add(&mut state.a, &state.d, &mut state.cc);
        }

        // ADD E
        0x83 => {
            instructions::add(&mut state.a, &state.e, &mut state.cc);
        }

        // ADD H
        0x84 => {
            instructions::add(&mut state.a, &state.h, &mut state.cc);
        }

        // ADD L
        0x85 => {
            instructions::add(&mut state.a, &state.l, &mut state.cc);
        }

        // ADD M
        0x86 => {
            let offset = (usize::from(state.h) << 8) | usize::from(state.l);
            instructions::add(&mut state.a, &state.memory[offset], &mut state.cc);
            /*let result: u16 = u16::from(state.a) + u16::from(state.memory[offset]);
            state.cc.cy = result > 0xff;
            state.a = result as u8;
            state.cc.z = state.a == 0;
            state.cc.s = state.a & 0x80 != 0;
            state.cc.p = parity(state.a);*/
        }

        // ADD A
        0x87 => {
            let a = state.a;
            instructions::add(&mut state.a, &a, &mut state.cc);
        }

        // SUB A
        0x97 => {
            let a = state.a;
            instructions::sub(&mut state.a, &a, &mut state.cc);
        }

        // ANA B
        0xa0 => {
            instructions::ana(&mut state.a, &state.b, &mut state.cc);
        }

        // ANA M
        0xa6 => {
            let offset = (usize::from(state.h) << 8) | usize::from(state.l);
            let m = state.memory[offset];
            instructions::ana(&mut state.a, &m, &mut state.cc);
        }

        // ANA A
        0xa7 => {
            let a = state.a;
            instructions::ana(&mut state.a, &a, &mut state.cc);
        }

        // XRA B
        0xa8 => instructions::xra(&mut state.a, &state.b, &mut state.cc),

        // XRA A
        0xaf => {
            let a = state.a;
            instructions::xra(&mut state.a, &a, &mut state.cc);
        }

        // ORA B
        0xb0 => {
            instructions::ora(&mut state.a, &state.b, &mut state.cc);
        }

        // ORA H
        0xb4 => instructions::ora(&mut state.a, &state.h, &mut state.cc),

        // ORA L
        0xb5 => instructions::ora(&mut state.a, &state.l, &mut state.cc),

        // ORA M
        0xb6 => {
            let offset = (usize::from(state.h) << 8) | usize::from(state.l);
            let m = state.memory[offset];
            instructions::ora(&mut state.a, &m, &mut state.cc);
        }

        // CMP B
        0xb8 => {
            instructions::cmp(&mut state.a, &state.b, &mut state.cc);
        }

        // CMP H
        0xbc => {
            instructions::cmp(&mut state.a, &state.h, &mut state.cc);
        }

        // CMP H
        0xbe => {
            let offset = (usize::from(state.h) << 8) | usize::from(state.l);
            let m = state.memory[offset];
            instructions::cmp(&mut state.a, &m, &mut state.cc);
        }

        // RNZ
        0xc0 => {
            if !state.cc.z {
                let sp: usize = usize::from(state.sp);
                state.pc = (u16::from(state.memory[sp + 1]) << 8) | u16::from(state.memory[sp]);
                state.sp += 2;
            } else {
                *cycles -= 6;
            }
        }

        // POP B
        0xc1 => {
            let sp: usize = usize::from(state.sp);
            state.c = state.memory[sp];
            state.b = state.memory[sp + 1];
            state.sp += 2;
        }

        // JNZ adr
        0xc2 => {
            if !state.cc.z {
                state.pc = (u16::from(state.memory[pc + 2]) << 8) | u16::from(state.memory[pc + 1]);
            } else {
            }
        }

        // JMP adr
        0xc3 => {
            state.pc = (u16::from(state.memory[pc + 2]) << 8) | u16::from(state.memory[pc + 1]);
        }

        // CNZ adr
        0xc4 => {
            if !state.cc.z {
                let sp: usize = usize::from(state.sp);
                //let ret = state.pc + 2; <-- maybe this is wrong
                let ret = state.pc;
                state.memory[sp - 1] = ret.to_be_bytes()[0];
                state.memory[sp - 2] = ret.to_be_bytes()[1];
                state.sp -= 2;
                state.pc = (u16::from(state.memory[pc + 2]) << 8) | u16::from(state.memory[pc + 1]);
            } else {
                *cycles -= 6;
            }
        }

        // PUSH B
        0xc5 => {
            let sp: usize = usize::from(state.sp);
            state.memory[sp - 1] = state.b;
            state.memory[sp - 2] = state.c;
            state.sp -= 2;
        }

        // ADI D8
        0xc6 => {
            let offset = usize::from(pc + 1);
            let result: u16 = u16::from(state.a) + u16::from(state.memory[offset]);
            state.cc.cy = result > 0xff;
            state.a = result as u8;
            state.cc.z = state.a == 0;
            state.cc.s = (state.a & 0x80) != 0;
            state.cc.p = parity(state.a);
        }

        // RZ
        0xc8 => {
            if state.cc.z {
                let sp: usize = usize::from(state.sp);
                state.pc = (u16::from(state.memory[sp + 1]) << 8) | u16::from(state.memory[sp]);
                state.sp += 2;
            } else {
                *cycles -= 6;
            }
        }

        // RET
        0xc9 => {
            let sp: usize = usize::from(state.sp);
            state.pc = (u16::from(state.memory[sp + 1]) << 8) | u16::from(state.memory[sp]);
            state.sp += 2;
        }

        // JZ adr
        0xca => {
            if state.cc.z {
                state.pc = (u16::from(state.memory[pc + 2]) << 8) | u16::from(state.memory[pc + 1]);
            } else {
            }
        }

        // CZ adr
        0xcc => {
            if state.cc.z {
                let sp: usize = usize::from(state.sp);
                let ret = state.pc;
                state.memory[sp - 1] = ret.to_be_bytes()[0];
                state.memory[sp - 2] = ret.to_be_bytes()[1];
                state.sp -= 2;
                state.pc = (u16::from(state.memory[pc + 2]) << 8) | u16::from(state.memory[pc + 1]);
            } else {
                *cycles -= 6;
            }
        }

        // CALL adr
        0xcd => {
            let sp: usize = usize::from(state.sp);
            let ret = state.pc;
            state.memory[sp - 1] = ret.to_be_bytes()[0];
            state.memory[sp - 2] = ret.to_be_bytes()[1];
            state.sp -= 2;
            state.pc = (u16::from(state.memory[pc + 2]) << 8) | u16::from(state.memory[pc + 1]);
        }

        // RNC
        0xd0 => {
            if !state.cc.cy {
                let sp: usize = usize::from(state.sp);
                state.pc = (u16::from(state.memory[sp + 1]) << 8) | u16::from(state.memory[sp]);
                state.sp += 2;
            } else {
                *cycles -= 6;
            }
        }

        // POP D
        0xd1 => {
            let sp: usize = usize::from(state.sp);
            state.e = state.memory[sp];
            state.d = state.memory[sp + 1];
            state.sp += 2;
        }

        // JNC adr
        0xd2 => {
            if !state.cc.cy {
                state.pc = (u16::from(state.memory[pc + 2]) << 8) | u16::from(state.memory[pc + 1]);
            } else {
            }
        }

        // OUT D8
        0xd3 => {
            special.machine_out(&state.memory[pc + 1], &state.a);
            /*println!("Instruction: {} op: {:x?} pc:{:x?}", total_instructions, opcode, pc);
            println!("a:{:x?} bc:{:x?}{:x?} de:{:x?}{:x?} hl:{:x?}{:x?} sp:{:x?}", state.a, state.b, state.c, state.d, state.e, state.h, state.l, state.sp);
            println!("cycles:{}", *cycles);
            dump_memory(&state);
            println!();*/
        }

        // CNC adr
        0xd4 => {
            if !state.cc.cy {
                let sp: usize = usize::from(state.sp);
                let ret = state.pc;
                state.memory[sp - 1] = ret.to_be_bytes()[0];
                state.memory[sp - 2] = ret.to_be_bytes()[1];
                state.sp -= 2;
                state.pc = (u16::from(state.memory[pc + 2]) << 8) | u16::from(state.memory[pc + 1]);
            } else {
                *cycles -= 6;
            }
        }

        // PUSH D
        0xd5 => {
            let sp: usize = usize::from(state.sp);
            state.memory[sp - 1] = state.d;
            state.memory[sp - 2] = state.e;
            state.sp -= 2;
        }

        // SUI D8
        0xd6 => {
            let offset = usize::from(pc + 1);
            //let a = u16::from(state.a) | 0x100; <-- uhmm.. what is this?
            let a = u16::from(state.a);
            let result: u16 = a.wrapping_sub(u16::from(state.memory[offset]));
            state.cc.cy = result >= 0x100;
            state.a = result.to_be_bytes()[1];
            state.cc.z = state.a == 0;
            state.cc.s = (state.a & 0x80) != 0;
            state.cc.p = parity(state.a);
        }

        // RC
        0xd8 => {
            if state.cc.cy {
                let sp: usize = usize::from(state.sp);
                state.pc = (u16::from(state.memory[sp + 1]) << 8) | u16::from(state.memory[sp]);
                state.sp += 2;
            } else {
                *cycles -= 6;
            }
        }

        // JC adr
        0xda => {
            if state.cc.cy {
                state.pc = (u16::from(state.memory[pc + 2]) << 8) | u16::from(state.memory[pc + 1]);
            } else {
            }
        }

        // IN D8
        0xdb => {
            state.a = special.machine_in(&state.memory[pc + 1], state);
        }

        //SBI D8
        0xde => {
            let a: u16 = u16::from(state.a);
            let result: u16 = a
                .wrapping_sub(u16::from(state.memory[pc + 1]))
                .wrapping_sub(u16::from(state.cc.cy));
            state.cc.cy = result >= 0x100;
            state.a = result.to_be_bytes()[1];
            let result = state.a;
            state.cc.z = result == 0;
            state.cc.s = (result & 0x80) != 0;
            state.cc.p = parity(result);
        }

        // POP H
        0xe1 => {
            let sp: usize = usize::from(state.sp);
            state.l = state.memory[sp];
            state.h = state.memory[sp + 1];
            state.sp += 2;
        }

        // XTHL
        0xe3 => {
            let bufferh = state.h;
            let bufferl = state.l;
            state.h = state.memory[usize::from(state.sp + 1)];
            state.l = state.memory[usize::from(state.sp)];
            state.memory[usize::from(state.sp + 1)] = bufferh;
            state.memory[usize::from(state.sp)] = bufferl;
        }

        // PUSH H
        0xe5 => {
            let sp: usize = usize::from(state.sp);
            state.memory[sp - 1] = state.h;
            state.memory[sp - 2] = state.l;
            state.sp -= 2;
        }

        // ANI D8
        0xe6 => {
            instructions::ana(&mut state.a, &state.memory[pc + 1], &mut state.cc);
        }

        // PCHL
        0xe9 => {
            state.pc = (u16::from(state.h) << 8) | u16::from(state.l);
        }

        // XCHG
        0xeb => {
            let bufferd = state.d;
            let buffere = state.e;
            state.d = state.h;
            state.e = state.l;
            state.h = bufferd;
            state.l = buffere;
        }

        // POP PSW
        //TODO : ÜBERPRÜFEN!
        0xf1 => {
            let sp = usize::from(state.sp);
            state.a = state.memory[sp + 1];
            let psw = state.memory[sp];
            state.cc.cy = (psw & 0b1) != 0;
            state.cc.p = (psw & 0b100) != 0;
            state.cc.ac = (psw & 0b10000) != 0;
            state.cc.z = (psw & 0b1000000) != 0;
            state.cc.s = (psw & 0b10000000) != 0;
            state.sp += 2;
        }

        // PUSH PSW
        0xf5 => {
            state.memory[usize::from(state.sp - 1)] = state.a;
            let mut psw: u8 = 0;
            if state.cc.cy {
                psw = psw | 0b1;
            }
            if state.cc.p {
                psw = psw | 0b100;
            }
            if state.cc.ac {
                psw = psw | 0b10000;
            }
            if state.cc.z {
                psw = psw | 0b1000000;
            }
            if state.cc.s {
                psw = psw | 0b10000000;
            }
            psw = psw | 0b10;
            state.memory[usize::from(state.sp - 2)] = psw;
            state.sp -= 2;
        }

        // TODO Besser machen
        // ORI D8
        0xf6 => {
            instructions::ora(&mut state.a, &state.memory[pc + 1], &mut state.cc);
            /*state.a = state.a | state.memory[pc + 1];
            state.cc.z = state.a == 0;
            state.cc.s = (state.a & 0x80) != 0;
            state.cc.cy = false;
            state.cc.p = parity(state.a);*/
        }

        // SPHL
        0xf9 => {
            state.sp = (u16::from(state.h) << 8) | u16::from(state.l);
        }

        // JM adr
        0xfa => {
            if state.cc.s {
                state.pc = (u16::from(state.memory[pc + 2]) << 8) | u16::from(state.memory[pc + 1]);
            }
        }

        // EI
        0xfb => {
            state.int_enable = true;
        }

        //CPI D8
        0xfe => {
            //let offset = usize::from(pc + 1);
            instructions::cmp(&mut state.a, &state.memory[pc + 1], &mut state.cc);
            /*let a = u16::from(state.a) | 0x100;
            let result: u16 = a - u16::from(state.memory[offset]);
            state.cc.cy = result < 0x100;
            let result = result as u8;
            state.cc.z = result == 0;
            state.cc.s = (result & 0x80) != 0;
            state.cc.p = parity(result);*/
        }

        _ => {
            unimplemented_instruction(opcode, pc);
            return false;
        }
    }

    /*for (i,item) in state.memory[0..0x3fff].iter().enumerate() {
        if state.memory[i] != memory[i] {
            println!("memory at addr: {:x?} changed {:x?} -> {:x?}", i, memory[i], state.memory[i]);
        }
    }
    println!(""); */

    return true;
}

fn parity(x: u8) -> bool {
    let mut one_bits: u8 = 0;
    for i in 0..8 {
        one_bits += (x >> i) & 0x1;
    }
    return (one_bits & 0x1) != 0;
}

fn unimplemented_instruction(opcode: u8, pc: usize) {
    println!(
        "Unimplemented Instruction: opcode: {:x?} pc: {:x?}",
        opcode, pc
    );
    //thread::sleep(time::Duration::from_secs(10));
    //process::exit(0x0);
}

fn generate_interrupt(state: &mut State8080, interrupt_type: &mut bool) {
    state.memory[usize::from(state.sp) - 1] = state.pc.to_be_bytes()[0];
    state.memory[usize::from(state.sp) - 2] = state.pc.to_be_bytes()[1];
    state.sp -= 2;
    state.pc = 8 * ((*interrupt_type as u16) + 1);
    state.int_enable = false;
}

fn dump_memory(state: &State8080) {
    let mut memString = String::new();
    for n in 0..16384 {
        if n % 16 == 0 {
            let mut hexString = format!("{:x?}", n);
            while (hexString.len() < 4) {
                hexString.insert(0, '0');
            }
            memString.push_str(format!("\n{}  ", hexString).as_str());
        }
        let mut tempString = format!("{:x?}", state.memory[n]);
        if (tempString.len() == 1) {
            tempString.insert(0, '0');
        }
        memString.push_str(&tempString.as_str());
    }
    println!("------memdump------");
    print!("{}", memString);
    exit(0);
}

struct Special {
    shift_offset: u8,
    shift0: u8,
    shift1: u8,
}
impl Special {
    fn machine_out(&mut self, port: &u8, value: &u8) {
        //println!("OUTPORT: {:?}", port);
        match port {
            2 => {
                self.shift_offset = *value & 0x7;
            }
            4 => {
                self.shift0 = self.shift1;
                self.shift1 = *value;
            }
            _ => {
                if *port != 3 && *port != 5 && *port != 6 {
                    println!("unimplemented special port(out): {:?}", port);
                }
                //exit(1);
            }
        }
    }
    fn machine_in(&mut self, port: &u8, state: &State8080) -> u8 {
        //println!("INPORT: {:?}", port);
        let mut a: u8 = 0;
        match port {
            3 => {
                let v: u16 = (u16::from(self.shift1) << 8) | u16::from(self.shift0);
                let buffer: u16 = (v >> (8 - self.shift_offset)) & 0xff;
                a = buffer.to_be_bytes()[1];
            }
            _ => {
                if *port != 1 && *port != 2 {
                    println!("unimplemented special port(in): {:?}", port);
                }
                //exit(1);
            }
        }
        return a;
    }
}

const CYCLES: [u8; 256] = [
    4,  10, 7,  5,  5,  5,  7,  4,  4,  10, 7,  5,  5,  5,  7,  4,
    4,  10, 7,  5,  5,  5,  7,  4,  4,  10, 7,  5,  5,  5,  7,  4,
    4,  10, 16, 5,  5,  5,  7,  4,  4,  10, 16, 5,  5,  5,  7,  4,
    4,  10, 13, 5,  10, 10, 10, 4,  4,  10, 13, 5,  5,  5,  7,  4,

    5,  5,  5,  5,  5,  5,  7,  5,  5,  5,  5,  5,  5,  5,  7,  5,
    5,  5,  5,  5,  5,  5,  7,  5,  5,  5,  5,  5,  5,  5,  7,  5,
    5,  5,  5,  5,  5,  5,  7,  5,  5,  5,  5,  5,  5,  5,  7,  5,
    7,  7,  7,  7,  7,  7,  7,  7,  5,  5,  5,  5,  5,  5,  7,  5,

    4,  4,  4,  4,  4,  4,  7,  4,  4,  4,  4,  4,  4,  4,  7,  4,
    4,  4,  4,  4,  4,  4,  7,  4,  4,  4,  4,  4,  4,  4,  7,  4,
    4,  4,  4,  4,  4,  4,  7,  4,  4,  4,  4,  4,  4,  4,  7,  4,
    4,  4,  4,  4,  4,  4,  7,  4,  4,  4,  4,  4,  4,  4,  7,  4,

    11, 10, 10, 10, 17, 11, 7,  11, 11, 10, 10, 10, 17, 17, 7,  11,
    11, 10, 10, 10, 17, 11, 7,  11, 11, 10, 10, 10, 17, 17, 7,  11,
    11, 10, 10, 18, 17, 11, 7,  11, 11, 5,  10, 5,  17, 17, 7,  11,
    11, 10, 10, 4,  17, 11, 7,  11, 11, 5,  10, 4,  17, 17, 7,  11
];

const SIZE: [u8; 256] = [
    // x1 x2 x3 x4 x5 x6 x7 x8 x9 xA xB xC xD xE xF
    1, 3, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, //0x
    1, 3, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, //1x
    1, 3, 3, 1, 1, 1, 2, 1, 1, 1, 3, 1, 1, 1, 2, 1, //2x
    1, 3, 3, 1, 1, 1, 2, 1, 1, 1, 3, 1, 1, 1, 2, 1, //3x
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, //4x
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, //5x
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, //6x
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, //7x
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, //8x
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, //9x
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, //Ax
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, //Bx
    1, 1, 3, 3, 3, 1, 2, 1, 1, 1, 3, 3, 3, 3, 2, 1, //Cx
    1, 1, 3, 2, 3, 1, 2, 1, 1, 1, 3, 2, 3, 3, 2, 1, //Dx
    1, 1, 3, 1, 3, 1, 2, 1, 1, 1, 3, 1, 3, 3, 2, 1, //Ex
    1, 1, 3, 1, 3, 1, 2, 1, 1, 1, 3, 1, 3, 3, 2, 1, //Fx
];
