#[allow(dead_code)]

//use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::process;
use std::num::Wrapping;

fn main() {
    static CYCLES_PER_FRAME: usize = 4_000_000 / 60;


    let condition = ConditionCodes {z:false, s:false, p:false, cy:false, ac:false, pad:0};
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
        println!("Interrup No: {}", i);
        if i > 100 { break; }
    }
}

fn emulate_instruction(state: &mut State8080, cycles: &mut usize) {
    let opcode: u8 = state.memory[usize::from(state.pc)];
    *cycles += usize::from(CYCLES[usize::from(opcode)]);
    //println!("Opcode: {:x?}", opcode);
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

        //0x02 => unimplemented_instruction(),

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
            let result = state.b.wrapping_sub(1);
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
            if state.e == 0 { state.d = 1; }
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
            let b: u8;
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
            state.l = state.l.wrapping_add(1);
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
        },

        // CMA (not)
        0x2f => {
            state.a = !state.a;
        },

        // LXI SP,word
        0x31 => {
            state.sp = (u16::from(state.memory[pc + 2]) << 8) | u16::from(state.memory[pc + 1]);
            state.pc += 2;
        },

        // STA adr
        0x32 => {
            let offset = (usize::from(state.memory[pc + 2]) << 8) | usize::from(state.memory[pc]);
            state.memory[offset] = state.a;
            state.pc += 2;
        },

        // DCR M
        0x035 => {
            let offset = (usize::from(state.h) << 8) | usize::from(state.l);
            state.memory[offset] -= 1;
            state.c = state.memory[offset];
            state.cc.z = state.memory[offset] == 0;
            state.cc.s = (state.memory[offset] & 0x80) != 0;
            state.cc.p = parity(state.memory[offset]);
        },

        // MVI M,D8
        0x36 => {
            let offset = (usize::from(state.h) << 8) | usize::from(state.l);
            state.memory[offset] = state.memory[pc + 1];
            state.pc += 1;
        },

        // STC
        0x37 => {
            state.cc.cy = true;
        },

        // LDA adr
        0x3a => {
            let offset = usize::from(state.memory[pc + 2]) << 8 | usize::from(state.memory[pc + 1]);
            state.a = state.memory[offset];
            state.pc += 2;
        },

        // DCR A
        0x3d => {
            state.a -= 1;
            state.cc.z = state.a == 0;
            state.cc.s = (state.a & 0x80) != 0;
            state.cc.p = parity(state.a);
        },

        // MVI A,D8
        0x3e => {
            state.a = state.memory[pc + 1];
            state.pc += 1;
        },

        // CMC
        0x3f => {
            state.cc.cy = !state.cc.cy;
        },

        // MOV B,B
        0x40 => {
            ()
        },

        // MOV B,C
        0x41 => {
            state.b = state.c
        },

        // MOV B,D
        0x42 => {
            state.b = state.d
        },

        // MOV B,E
        0x43 => {
            state.b = state.e
        },

        // MOV B,H
        0x44 => {
            state.b = state.h
        },

        // MOV B,L
        0x45 => {
            state.b = state.l
        },

        // MOV B,M
        0x46 => {
            let offset = (usize::from(state.h) << 8) | usize::from(state.l);
            state.b = state.memory[offset];
        },

        // MOV C,A
        0x4f => {
            state.c = state.a;
        },

        // MOV D,M
        0x56 => {
            let offset = (usize::from(state.h) << 8) | usize::from(state.l);
            state.d = state.memory[offset];
        },

        // MOV D,A
        0x57 => {
            state.d = state.a;
        },

        // MOV E,M
        0x5e => {
            let offset = (usize::from(state.h) << 8) | usize::from(state.l);
            state.e = state.memory[offset];
        },

        // MOV E,A
        0x5f => {
            state.e = state.a;
        },

        // MOV H,M
        0x66 => {
            let offset = (usize::from(state.h) << 8) | usize::from(state.l);
            state.h = state.memory[offset];
        },

        // MOV H,A
        0x67 => {
            state.h = state.a;
        },

        // MOV L,A
        0x6f => {
            state.l = state.a;
        },

        // MOV M,B
        0x70 => {
            let offset = (usize::from(state.h) << 8) | usize::from(state.l);
            state.memory[offset] = state.b;
        },

        // MOV M,A
        0x77 => {
            let offset = (usize::from(state.h) << 8) | usize::from(state.l);
            state.memory[offset] = state.a;
        },

        // MOV A,B
        0x78 => {
            state.a = state.b;
        },

        // MOV A,C
        0x79 => {
            state.a = state.c;
        },

        // MOV A,D
        0x7a => {
            state.a = state.d;
        },

        // MOV A,E
        0x7b => {
            state.a = state.e;
        },

        // MOV A,H
        0x7c => {
            state.a = state.h;
        },

        // MOV A,L
        0x7d => {
            state.a = state.l;
        },

        // MOV A,M
        0x7e => {
            let offset = (usize::from(state.h) << 8) | usize::from(state.l);
            state.a = state.memory[offset];
        },

        // ADD B
        0x80 => {
            let result: u16 = u16::from(state.a) + u16::from(state.b);
            state.cc.cy = result > 0xff;
            state.a += state.b;
            state.cc.z = state.a == 0;
            state.cc.s = state.a & 0x80 != 0;
            state.cc.p = parity(state.a);
        },

        // ADD C
        0x81 => {
            let result: u16 = u16::from(state.a) + u16::from(state.c);
            state.cc.cy = result > 0xff;
            state.a += state.c;
            state.cc.z = state.a == 0;
            state.cc.s = state.a & 0x80 != 0;
            state.cc.p = parity(state.a);
        },

        // ADD D
        0x82 => {
            let result: u16 = u16::from(state.a) + u16::from(state.d);
            state.cc.cy = result > 0xff;
            state.a += state.d;
            state.cc.z = state.a == 0;
            state.cc.s = state.a & 0x80 != 0;
            state.cc.p = parity(state.a);
        },

        // ADD E
        0x83 => {
            let result: u16 = u16::from(state.a) + u16::from(state.e);
            state.cc.cy = result > 0xff;
            state.a += state.e;
            state.cc.z = state.a == 0;
            state.cc.s = state.a & 0x80 != 0;
            state.cc.p = parity(state.a);
        },

        // ADD H
        0x84 => {
            let result: u16 = u16::from(state.a) + u16::from(state.h);
            state.cc.cy = result > 0xff;
            state.a += state.h;
            state.cc.z = state.a == 0;
            state.cc.s = state.a & 0x80 != 0;
            state.cc.p = parity(state.a);
        },

        // ADD L
        0x85 => {
            let result: u16 = u16::from(state.a) + u16::from(state.l);
            state.cc.cy = result > 0xff;
            state.a += state.l;
            state.cc.z = state.a == 0;
            state.cc.s = state.a & 0x80 != 0;
            state.cc.p = parity(state.a);
        },

        // ADD M
        0x86 => {
            let offset = (usize::from(state.h) << 8) | usize::from(state.l);
            let result: u16 = u16::from(state.a) + u16::from(state.memory[offset]);
            state.cc.cy = result > 0xff;
            state.a += state.memory[offset];
            state.cc.z = state.a == 0;
            state.cc.s = state.a & 0x80 != 0;
            state.cc.p = parity(state.a);
        },

        // ADD A
        0x87 => {
            let result: u16 = u16::from(state.a) * 2;
            state.cc.cy = result > 0xff;
            state.a += state.b;
            state.cc.z = state.a == 0;
            state.cc.s = state.a & 0x80 != 0;
            state.cc.p = parity(state.a);
        },

        // ANA A
        0xa7 => {
            state.cc.z = state.a == 0;
            state.cc.s = (state.a & 0x80) != 0;
            state.cc.p = parity(state.a);
        },

        // XRA A
        0xaf => {
            state.a = 0x0;
            state.cc.z = true;
            state.cc.s = false;
            state.cc.cy = false;
            state.cc.p = true;
        },

        // ORA B
        0xb0 => {
            state.a = state.a | state.b;
            state.cc.z = state.a == 0;
            state.cc.s = (state.a & 0x80) != 0;
            state.cc.cy = false;
            state.cc.p = parity(state.a);
        },

        // ORA M
        0xb6 => {
            let offset = (usize::from(state.h) << 8) | usize::from(state.l);
            state.a = state.a | state.memory[offset];
            state.cc.z = state.a == 0;
            state.cc.s = (state.a & 0x80) != 0;
            state.cc.cy = false;
            state.cc.p = parity(state.a);
        },

        // RNZ
        0xc0 => {
            if !state.cc.z {
                let sp: usize = usize::from(state.sp);
                state.pc = (u16::from(state.memory[sp + 1]) << 8) | u16::from(state.memory[sp]);
                state.sp += 2;
            }
        },

        // POP B
        0xc1 => {
            let sp: usize = usize::from(state.sp);
            state.c = state.memory[sp];
            state.b = state.memory[sp + 1];
            state.sp += 2;
        },

        // JNZ adr
        0xc2 => {
            if !state.cc.z {
                state.pc = (u16::from(state.memory[pc + 2]) << 8) | u16::from(state.memory[pc + 1]);
            } else {
                state.pc += 2;
            }
        },

        // JMP adr
        0xc3 => {
            state.pc = (u16::from(state.memory[pc + 2]) << 8) | u16::from(state.memory[pc + 1]);
        },

        // CNZ adr
        0xc4 => {
            if !state.cc.z {
                let sp: usize = usize::from(state.sp);
                let ret = state.pc + 2;
                state.memory[sp - 1] = ret.to_be_bytes()[0];
                state.memory[sp - 2] = ret.to_be_bytes()[1];
                state.sp -= 2;
                state.pc = (u16::from(state.memory[pc + 2]) << 8) | u16::from(state.memory[pc + 1]);
            } else {
                state.pc += 2;
            }
        },

        // PUSH B
        0xc5 => {
            let sp: usize = usize::from(state.sp);
            state.memory[sp - 1] = state.b;
            state.memory[sp - 2] = state.c;
            state.sp -= 2;
        },

        // ADI D8
        0xc6 => {
            let offset = usize::from(pc + 1);
            let result: u16 = u16::from(state.a) + u16::from(state.memory[offset]);
            state.cc.cy = result > 0xff;
            state.a += state.memory[offset];
            state.cc.z = state.a == 0;
            state.cc.s = (state.a & 0x80) != 0;
            state.cc.p = parity(state.a);
            state.pc += 1;
        },

        // RZ
        0xc8 => {
            if state.cc.z {
                let sp: usize = usize::from(state.sp);
                state.pc = (u16::from(state.memory[sp + 1]) << 8) | u16::from(state.memory[sp]);
                state.sp += 2;
            }
        },

        // RET
        0xc9 => {
            let sp: usize = usize::from(state.sp);
            state.pc = (u16::from(state.memory[sp + 1]) << 8) | u16::from(state.memory[sp]);
        },

        // JZ adr
        0xca => {
            if state.cc.z {
                state.pc = (u16::from(state.memory[pc + 2]) << 8) | u16::from(state.memory[pc + 1]);
            } else {
                state.pc += 2;
            }
        },

        // CALL adr
        0xcd => {
            let sp: usize = usize::from(state.sp);
            let ret = state.pc + 2;
            state.memory[sp - 1] = ret.to_be_bytes()[0];
            state.memory[sp - 2] = ret.to_be_bytes()[1];
            state.sp -= 2;
            state.pc = (u16::from(state.memory[pc + 2]) << 8) | u16::from(state.memory[pc + 1]);
        },

        // RNC
        0xd0 => {
            if !state.cc.cy {
                let sp: usize = usize::from(state.sp);
                state.pc = (u16::from(state.memory[sp + 1]) << 8) | u16::from(state.memory[sp]);
                state.sp += 2;
            }
        },

        // POP D
        0xd1 => {
            let sp: usize = usize::from(state.sp);
            state.e = state.memory[sp];
            state.d = state.memory[sp + 1];
            state.sp += 2;
        },

        // JNC adr
        0xd2 => {
            if !state.cc.cy {
                state.pc = (u16::from(state.memory[pc + 2]) << 8) | u16::from(state.memory[pc + 1]);
            } else {
                state.pc += 2;
            }
        },

        // OUT D8
        0xd3 => {
            //TODO: Implement machine out
            state.pc += 1;
        },

        // PUSH D
        0xd5 => {
            let sp: usize = usize::from(state.sp);
            state.memory[sp - 1] = state.d;
            state.memory[sp - 2] = state.e;
            state.sp -= 2;
        },

        // SUI D8
        0xd6 => {
            let offset = usize::from(pc + 1);
            let result: u16 = u16::from(state.a) - u16::from(state.memory[offset]);
            state.cc.cy = result > 0xff;
            state.a -= state.memory[offset];
            state.cc.z = state.a == 0;
            state.cc.s = (state.a & 0x80) != 0;
            state.cc.p = parity(state.a);
            state.pc += 1;
        },

        // RC
        0xd8 => {
            if state.cc.cy {
                let sp: usize = usize::from(state.sp);
                state.pc = (u16::from(state.memory[sp + 1]) << 8) | u16::from(state.memory[sp]);
                state.sp += 2;
            }
        },

        // JC adr
        0xda => {
            if state.cc.cy {
                state.pc = (u16::from(state.memory[pc + 2]) << 8) | u16::from(state.memory[pc + 1]);
            } else {
                state.pc += 2;
            }
        },

        // IN D8
        0xdb => {
            //TODO: Implement machine out
            state.pc += 1;
        },

        // POP H
        0xe1 => {
            let sp: usize = usize::from(state.sp);
            state.h = state.memory[sp];
            state.l = state.memory[sp + 1];
            state.sp += 2;
        },

        // XTHL
        0xe3 => {
            let bufferh = state.h;
            let bufferl = state.l;
            state.h = state.memory[usize::from(state.sp + 1)];
            state.l = state.memory[usize::from(state.sp)];
            state.memory[usize::from(state.sp + 1)] = bufferh;
            state.memory[usize::from(state.sp)] = bufferl;
        },

        // PUSH H
        0xe5 => {
            let sp: usize = usize::from(state.sp);
            state.memory[sp - 1] = state.h;
            state.memory[sp - 2] = state.l;
            state.sp -= 2;
        },

        // ANI D8
        0xe6 => {
            state.a = state.a & state.memory[pc + 1];
            state.cc.z = state.a == 0;
            state.cc.s = (state.a & 0x80) != 0;
            state.cc.p = parity(state.a);
            state.cc.cy = false;
            state.pc += 1;
        },

        // PCHL
        0xe9 => {
            state.pc = (u16::from(state.h) << 8) | u16::from(state.l);
        },

        // XCHG
        0xeb => {
            let bufferd = state.d;
            let buffere = state.e;
            state.d = state.h;
            state.e = state.l;
            state.h = bufferd;
            state.l = buffere;
        },

        // POP PSW
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
        },

        // PUSH PSW
        0xf5 => {
            state.memory[usize::from(state.sp - 1)] = state.a;
            let mut psw: u8 = 0;
            if state.cc.cy { psw = psw | 0b1; }
            if state.cc.p { psw = psw | 0b100; }
            if state.cc.ac { psw = psw | 0b10000; }
            if state.cc.z { psw = psw | 0b1000000; }
            if state.cc.s { psw = psw | 0b10000000; }
            psw = psw | 0b10;
            state.memory[usize::from(state.sp - 2)] = psw;
            state.sp -= 2;
        },

        // ORI D8
        0xf6 => {
            state.a = state.a | state.memory[pc + 1];
            state.cc.z = state.a == 0;
            state.cc.s = (state.a & 0x80) != 0;
            state.cc.cy = false;
            state.cc.p = parity(state.a);
            state.pc += 1;
        },

        // SPHL
        0xf9 => {
            state.sp = (u16::from(state.h) << 8) | u16::from(state.l);
        },

        // EI
        0xfb => {
            state.int_enable = true;
        },

        //CPI D8
        0xfe => {
            let result = u16::from(state.a) - u16::from(state.memory[pc + 1]);
            state.cc.cy = result > 0xff;
            let result = state.a - state.memory[pc + 1];
            state.cc.z = result == 0;
            state.cc.s = (result & 0x80) != 0;
            state.cc.p = parity(result);
            state.pc += 1;
        },



        _ => {unimplemented_instruction(opcode)},
    }
}

fn parity(x: u8) -> bool {
    let mut one_bits: u8 = 0;
    for i in 0..8 {
        one_bits += (x >> i) & 0x1;
    }
    return (one_bits & 0x1) != 0;
}

fn unimplemented_instruction(opcode: u8) {
    println!("Unimplemented Instruction: {:x?}", opcode);
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
    ac: bool,
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