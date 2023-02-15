use std::{fs::File, io::Read, ops::RangeInclusive, process::exit};

const CYCLES_PER_FRAME: isize = 2_000_000 / 60;

/** aka the flags register */
pub struct ConditionCodes {
    /** bit 7; Sign Flag */
    pub s: bool,
    /** Bit 6; zero flag */
    pub z: bool,
    /** Bit 5; always 0 */
    pub five: bool,
    /** Bit 4; Auxiliary Carry Flag */
    pub ac: bool,
    /** Bit 3; always 0 */
    pub three: bool,
    /** Bit 2; Parity Flag */
    pub p: bool,
    /** Bit 1; always 1 */
    pub one: bool,
    /** Bit 0; Carry Flag */
    pub cy: bool,
}

impl ConditionCodes {
    pub fn set_arithmetic_flags(&mut self, val: &u16, do_cy: bool) {
        self.z = (*val & 0xff) == 0;
        self.s = 0x80 == (*val & 0x80);
        self.p = (*val & 0xff).count_ones() % 2 == 0;
        if do_cy {
            self.cy = *val >> 8 != 0;
        }
        self.ac = *val >> 8 != 0;
    }

    pub fn set_zsp(&mut self, val: &u16) {
        self.z = (*val & 0xff) == 0;
        self.s = 0x80 == (*val & 0x80);
        self.p = (*val & 0xff).count_ones() % 2 == 0;
    }

    pub fn get_f(&self) -> u8 {
        return (u8::from(self.s) << 7)
            | (u8::from(self.z) << 6)
            | (u8::from(self.five) << 5)
            | (u8::from(self.ac) << 4)
            | (u8::from(self.three) << 3)
            | (u8::from(self.p) << 2)
            | (u8::from(self.one) << 1)
            | (u8::from(self.cy));
    }

    pub fn set_f(&mut self, val: u8) {
        self.s = (val >> 7) == 0x1;
        self.z = ((val >> 6) & 0x1) == 0x1;
        self.ac = ((val >> 4) & 0x1) == 0x1;
        self.p = ((val >> 2) & 0x1) == 0x1;
        self.cy = (val & 0x1) == 0x1;
    }

    fn carry(&mut self, bit_no: u8, a: u8, b: u8, cy: bool) -> bool {
        let result: u16 = a as u16 + b as u16 + cy as u16;
        let carry: u16 = result ^ a as u16 ^ b as u16;
        return (carry & (1 << bit_no as u16)) != 0;
    }

    fn set_cy(&mut self, a: u8, b: u8, cy: bool) {
        self.cy = self.carry(8, a, b, cy);
    }

    fn set_ac(&mut self, a: u8, b: u8, cy: bool) {
        self.ac = self.carry(4, a, b, cy);
    }
}

pub struct State8080 {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,

    pub sp: u16,
    pub pc: u16,

    pub memory: Memory,

    pub cc: ConditionCodes,

    pub int_enable: bool,
    pub int_type: bool,
    pub int_vector: u8,
    pub int_delay: u8,

    pub cycles: isize,

    pub special: Special,
}

impl State8080 {
    fn get_af(&self) -> u16 {
        return (u16::from(self.a) << 8) | u16::from(self.cc.get_f());
    }

    fn set_af(&mut self, val: u16) {
        let le_bytes = val.to_le_bytes();
        self.a = le_bytes[1];
        self.cc.set_f(le_bytes[1]);
    }

    fn get_bc(&self) -> u16 {
        return (u16::from(self.b) << 8) | u16::from(self.c);
    }

    fn set_bc(&mut self, val: u16) {
        let le_bytes = val.to_le_bytes();
        self.b = le_bytes[1];
        self.c = le_bytes[0];
    }

    fn get_de(&self) -> u16 {
        return (u16::from(self.d) << 8) | u16::from(self.e);
    }

    fn set_de(&mut self, val: u16) {
        let le_bytes = val.to_le_bytes();
        self.d = le_bytes[1];
        self.e = le_bytes[0];
    }

    pub fn get_hl(&self) -> u16 {
        return (u16::from(self.h) << 8) | u16::from(self.l);
    }

    fn set_hl(&mut self, val: u16) {
        let le_bytes = val.to_le_bytes();
        self.h = le_bytes[1];
        self.l = le_bytes[0];
    }

    pub fn load_rom(&mut self, range: RangeInclusive<usize>, path: &str) {
        let mut invadersh: File = File::open(path).expect("no such file");
        invadersh
            .read(&mut self.memory.memory[range])
            .expect("error reading into emulated memory");
    }

    fn read_next(&mut self) -> u8 {
        let result = self.memory.read(self.pc);
        self.pc += 1;
        return result;
    }

    fn read_next_word(&mut self) -> u16 {
        let result = self.memory.read_word(self.pc);
        self.pc += 2;
        return result;
    }

    fn read_next_word_as_u8(&mut self, h: &mut u8, l: &mut u8) {
        self.memory.read_word_as_u8(self.pc, h, l);
        self.pc += 2;
    }

    fn write_next(&mut self, val: u8) -> bool {
        let result = self.memory.write(self.pc, val);
        self.pc += 1;
        return result;
    }

    fn write_next_word(&mut self, val_h: u8, val_l: u8) -> bool {
        let result = self.memory.write_word(self.pc, val_h, val_l);
        self.pc += 2;
        return result;
    }

    fn write_next_word_with_u16(&mut self, val: u16) -> bool {
        let result = self.memory.write_word_with_u16(self.pc, val);
        self.pc += 2;
        return result;
    }

    fn nop(&self) {}

    fn inr(&mut self, register: u8) -> u8 {
        let result = register.wrapping_add(1);
        let val = u16::from(register) + 1;
        self.cc.set_zsp(&val);
        self.cc.ac = (register & 0xf) == 0;
        return result;
    }

    /** returns the modified variable val, not needed anymore */
    fn inr_m(&mut self, val: u8) -> u8 {
        return self.inr(val);
    }

    fn dcr(&mut self, register: u8) -> u8 {
        let result = register.wrapping_sub(1);
        let val = u16::from(register).wrapping_sub(1);
        self.cc.set_zsp(&val);
        self.cc.ac = !((register & 0xf) == 0);
        return result;
    }

    /** returns the modified variable val, not needed anymore */
    fn dcr_m(&mut self, val: u8) -> u8 {
        return self.dcr(val);
    }

    fn rlc(&mut self) {
        self.cc.cy = (self.a >> 7) != 0;
        self.a = (self.a >> 7) | (self.a << 1);
    }

    fn rrc(&mut self) {
        self.cc.cy = (self.a & 0x1) != 0;
        self.a = (self.a << 7) | (self.a >> 1);
    }

    fn rar(&mut self) {
        let cy: bool = self.cc.cy;
        self.cc.cy = self.a & 1 != 0;
        self.a = (self.a >> 1) | ((cy as u8) << 7);
    }

    fn dad(&mut self, val: u16) {
        let result = u32::from(self.get_hl()) + u32::from(val);
        self.cc.cy = (result & 0xffff0000) > 0;
        let le_bytes = result.to_le_bytes();
        self.h = le_bytes[1];
        self.l = le_bytes[0];
    }

    fn add_adc(&mut self, sregister: u8, cy: bool) {
        let result: u16 = u16::from(self.a) + u16::from(sregister) + cy as u16;
        self.a = (result & 0xff) as u8;
        self.cc.set_zsp(&result);
        self.cc.set_cy(self.a, sregister, cy);
        self.cc.set_ac(self.a, sregister, cy);
    }

    fn add(&mut self, sregister: u8) {
        self.add_adc(sregister, false);
    }

    fn adc(&mut self, sregister: u8) {
        self.add_adc(sregister, self.cc.cy);
    }

    fn sub(&mut self, sregister: u8) {
        self.add_adc(!sregister, false);
        self.cc.cy = !self.cc.cy
    }

    fn sbb(&mut self, sregister: u8) {
        self.add_adc(!sregister, self.cc.cy);
        self.cc.cy = !self.cc.cy
    }

    fn ana(&mut self, val: u8) {
        let result = self.a & val;
        self.cc.cy = false;
        self.cc.ac = ((self.a | val) & 0x08) != 0;
        self.cc.set_zsp(&(result as u16));
        self.a = result;
    }

    fn xra(&mut self, val: u8) {
        self.a ^= val;
        self.cc.cy = false;
        self.cc.ac = false;
        self.cc.set_zsp(&(self.a as u16));
    }

    fn ora(&mut self, val: u8) {
        self.a |= val;
        self.cc.cy = false;
        self.cc.ac = false;
        self.cc.set_zsp(&(self.a as u16));
    }

    fn cmp(&mut self, val: u8) {
        let result = u16::from(self.a).wrapping_sub(u16::from(val));
        self.cc.cy = result >> 8 > 0;
        self.cc.ac = !(u16::from(self.a) ^ result ^ u16::from(val)) & 0x10 > 0;
        self.cc.set_zsp(&result);
    }

    fn push(&mut self, val: u16) {
        self.sp -= 2;
        self.memory.write_word_with_u16(self.sp, val);
    }

    fn pop(&mut self) -> u16 {
        let val = self.memory.read_word(self.sp);
        self.sp += 2;
        return val;
    }

    fn ret(&mut self) {
        self.pc = self.pop();
    }

    fn cond_ret(&mut self, cond: bool) {
        if cond {
            self.ret();
        }
    }

    fn jump(&mut self, index: u16) {
        self.pc = index;
    }

    fn cond_jump(&mut self, cond: bool) {
        let index: u16 = self.read_next_word();
        if cond {
            self.jump(index);
        }
    }

    fn call(&mut self, index: u16) {
        self.push(self.pc);
        self.jump(index);
    }

    fn cond_call(&mut self, cond: bool) {
        let index: u16 = self.read_next_word();
        if cond {
            self.call(index);
        }
    }

    fn push_psw(&mut self) {
        let val = (u16::from(self.a) << 8) | u16::from(self.cc.get_f());
        self.push(val);
    }

    fn pop_psw(&mut self) {
        let val = self.pop();
        self.a = (val >> 8) as u8;
        let psw = (val & 0xff) as u8;
        self.cc.set_f(psw);
    }

    fn xthl(&mut self) {
        let val = self.memory.read_word(self.sp);
        self.memory.write_word(self.sp, self.h, self.l);
        self.set_hl(val);
    }

    fn xchg(&mut self) {
        let de = self.get_de();
        self.set_de(self.get_hl());
        self.set_hl(de);
    }

    fn interrupt(&mut self) {
        self.push(self.pc);
        self.pc = 8 * ((self.int_type as u16) + 1);
        self.int_enable = false;
        self.int_type = !self.int_type;
        self.cycles -= CYCLES_PER_FRAME / 2;
    }

    pub fn step(&mut self) -> bool {
        if self.cycles < CYCLES_PER_FRAME / 2 {
            let opcode = self.read_next();
            let result = self.do_instruction(opcode);
            if !result {
                println!("Error or unimplemented Instruction. Opcode: {:x?}", opcode);
                self.dump_memory();
                exit(1);
            }
            self.cycles += isize::from(CYCLES[usize::from(opcode)]);
            return false;
        } else {
            self.interrupt();
            return true;
        }
    }

    fn do_instruction(&mut self, opcode: u8) -> bool {
        let mut ret = true;
        match opcode {
            // NOP
            0x00 => self.nop(),
            // LXI
            //0x01 => self.read_next_word_as_u8(&mut &self.b, &mut self.c),
            0x01 => {
                let val = self.read_next_word();
                self.set_bc(val);
            }
            //0x11 => self.read_next_word_as_u8(&mut self.d, &mut self.e),
            0x11 => {
                let val = self.read_next_word();
                self.set_de(val);
            }
            //0x21 => self.read_next_word_as_u8(&mut self.h, &mut self.l),
            0x21 => {
                let val = self.read_next_word();
                self.set_hl(val);
            }
            0x31 => self.sp = self.read_next_word(),
            // INX
            0x03 => self.set_bc(self.get_bc().wrapping_add(1)),
            0x13 => self.set_de(self.get_de().wrapping_add(1)),
            0x23 => self.set_hl(self.get_hl().wrapping_add(1)),
            // DCX
            0x2b => self.set_hl(self.get_hl().wrapping_sub(1)),
            // INR
            0x04 => self.b = self.inr(self.b),
            0x0c => self.c = self.inr(self.c),
            0x14 => self.d = self.inr(self.d),
            0x2c => self.l = self.inr(self.l),
            0x34 => {
                let val = self.inr(self.memory.read(self.get_hl()));
                ret = self.memory.write(self.get_hl(), val);
            }
            0x3c => self.a = self.inr(self.a),
            // DCR
            0x05 => self.b = self.dcr(self.b),
            0x0d => self.c = self.dcr(self.c),
            0x15 => self.d = self.dcr(self.d),
            0x35 => {
                let val = self.inr(self.memory.read(self.get_hl()));
                ret = self.memory.write(self.get_hl(), val)
            }
            0x3d => self.a = self.dcr(self.a),
            // MVI
            0x06 => self.b = self.read_next(),
            0x0e => self.c = self.read_next(),
            0x16 => self.d = self.read_next(),
            0x26 => self.h = self.read_next(),
            0x2e => self.l = self.read_next(),
            0x36 => {
                let val = self.read_next();
                ret = self.memory.write(self.get_hl(), val);
            }
            0x3e => self.a = self.read_next(),
            // else
            0x07 => self.rlc(),
            0x0f => self.rrc(),
            0x1f => self.rar(),
            0x2f => self.a = !self.a,
            0x37 => self.cc.cy = true,
            0x3f => self.cc.cy = !self.cc.cy,
            0xe9 => self.pc = self.get_hl(),
            0xf9 => self.sp = self.get_hl(),
            0xfb => self.int_enable = true,
            // DAD
            0x09 => self.dad(self.get_bc()),
            0x19 => self.dad(self.get_de()),
            0x29 => self.dad(self.get_hl()),
            // LDAX LHLD LDA
            0x0a => self.a = self.memory.read(self.get_bc()),
            0x1a => self.a = self.memory.read(self.get_de()),
            //0x2a => self.read_next_word_as_u8(&mut self.h, &mut self.l),
            0x2a => {
                let val = self.read_next_word();
                self.set_hl(val)
            }
            0x3a => {
                let val = self.read_next_word();
                self.a = self.memory.read(val);
            }
            // STAX SHLD STA
            0x12 => ret = self.memory.write(self.get_de(), self.a),
            0x22 => ret = self.write_next_word(self.h, self.l),
            0x32 => {
                let val = self.read_next_word();
                ret = self.memory.write(val, self.a)
            }
            // MOV
            0x40 => self.nop(),
            0x41 => self.b = self.c,
            0x42 => self.b = self.d,
            0x43 => self.b = self.e,
            0x44 => self.b = self.h,
            0x45 => self.b = self.l,
            0x46 => self.b = self.memory.read(self.get_hl()),
            0x47 => self.b = self.a,
            0x4e => self.c = self.memory.read(self.get_hl()),
            0x4f => self.c = self.a,
            0x56 => self.d = self.memory.read(self.get_hl()),
            0x57 => self.d = self.a,
            0x5e => self.e = self.memory.read(self.get_hl()),
            0x5f => self.e = self.a,
            0x61 => self.h = self.c,
            0x65 => self.h = self.l,
            0x66 => self.h = self.memory.read(self.get_hl()),
            0x67 => self.h = self.a,
            0x68 => self.l = self.b,
            0x69 => self.l = self.c,
            0x6f => self.l = self.a,
            0x70 => ret = self.memory.write(self.get_hl(), self.b),
            0x71 => ret = self.memory.write(self.get_hl(), self.c),
            0x77 => ret = self.memory.write(self.get_hl(), self.a),
            0x78 => self.a = self.b,
            0x79 => self.a = self.c,
            0x7a => self.a = self.d,
            0x7b => self.a = self.e,
            0x7c => self.a = self.h,
            0x7d => self.a = self.l,
            0x7e => self.a = self.memory.read(self.get_hl()),
            // ADD
            0x80 => self.add(self.b),
            0x81 => self.add(self.c),
            0x82 => self.add(self.d),
            0x83 => self.add(self.e),
            0x84 => self.add(self.h),
            0x85 => self.add(self.l),
            0x86 => self.add(self.memory.read(self.get_hl())),
            0x87 => self.add(self.a),
            0xc6 => {
                let val = self.read_next();
                self.add(val);
            }
            // SUB
            0x97 => self.sub(self.a),
            0xd6 => {
                let val = self.read_next();
                self.sub(val);
            }
            // SBB
            0xde => {
                let val = self.read_next();
                self.sbb(val);
            }
            // ANA
            0xa0 => self.ana(self.b),
            0xa6 => self.ana(self.memory.read(self.get_hl())),
            0xa7 => self.ana(self.a),
            0xe6 => {
                let val = self.read_next();
                self.ana(val);
            }
            // XRA
            0xa8 => self.xra(self.b),
            0xaf => self.xra(self.a),
            // ORA
            0xb0 => self.ora(self.b),
            0xb4 => self.ora(self.h),
            0xb5 => self.ora(self.l),
            0xb6 => self.ora(self.memory.read(self.get_hl())),
            0xf6 => {
                let val = self.read_next();
                self.ora(val);
            }
            // CMP
            0xb8 => self.cmp(self.b),
            0xbc => self.cmp(self.h),
            0xbe => {
                let val = self.memory.read(self.get_hl());
                self.cmp(val);
            }
            0xfe => {
                let val = self.read_next();
                self.cmp(val);
            }
            // RETs
            0xc0 => self.cond_ret(!self.cc.z),
            0xc8 => self.cond_ret(self.cc.z),
            0xc9 => self.ret(),
            0xd0 => self.cond_ret(!self.cc.cy),
            0xd8 => self.cond_ret(self.cc.cy),
            // PUSH
            0xc5 => self.push(self.get_bc()),
            0xd5 => self.push(self.get_de()),
            0xe5 => self.push(self.get_hl()),
            0xf5 => self.push_psw(),
            // POP
            0xc1 => {
                let pop = self.pop();
                self.set_bc(pop);
            }
            0xd1 => {
                let pop = self.pop();
                self.set_de(pop);
            }
            0xe1 => {
                let pop = self.pop();
                self.set_hl(pop);
            }
            0xf1 => self.pop_psw(),
            // JMP
            0xc2 => self.cond_jump(!self.cc.z),
            0xc3 => {
                let val = self.read_next_word();
                self.jump(val);
            }
            0xca => self.cond_jump(self.cc.z),
            0xd2 => self.cond_jump(!self.cc.cy),
            0xda => self.cond_jump(self.cc.cy),
            0xfa => self.cond_jump(self.cc.s),
            // CALL
            0xc4 => self.cond_call(!self.cc.z),
            0xcc => self.cond_call(self.cc.z),
            0xcd => {
                let val = self.read_next_word();
                self.call(val);
            }
            0xd4 => self.cond_call(!self.cc.cy),
            // IN/OUT
            0xd3 => {
                let val = self.read_next();
                self.special.machine_out(val, self.a);
            }
            0xdb => {
                let val = self.read_next();
                self.a = self.special.machine_in(val);
            }
            // XTHL XCHG
            0xe3 => self.xthl(),
            0xeb => self.xchg(),

            _ => return false,
        }
        return ret;
    }

    pub fn dump_memory(&self) {
        let mut mem_string = String::new();
        for n in 0..16384 {
            if n % 16 == 0 {
                let mut hex_string = format!("{:x?}", n);
                while hex_string.len() < 4 {
                    hex_string.insert(0, '0');
                }
                mem_string.push_str(format!("\n{}  ", hex_string).as_str());
            }
            let mut temp_string = format!("{:x?}", self.memory.memory[n]);
            if (temp_string.len() == 1) {
                temp_string.insert(0, '0');
            }
            mem_string.push_str(&temp_string.as_str());
        }
        println!("pc:{:x?}", self.pc);
        println!(
            "a:{:x?} bc:{:x?}{:x?} de:{:x?}{:x?} hl:{:x?}{:x?} sp:{:x?}",
            self.a, self.b, self.c, self.d, self.e, self.h, self.l, self.sp
        );
        println!("cycles:{}", self.cycles);
        println!("------memdump------");
        print!("{}", mem_string);
        exit(0);
    }
}

pub struct Register {
    pub register: u8,
}

pub struct Memory {
    pub memory: [u8; 16384],
}

impl Memory {
    fn read(&self, index: u16) -> u8 {
        return self.memory[usize::from(index)];
    }

    fn read_word(&self, index: u16) -> u16 {
        return (u16::from(self.read(index + 1)) << 8) | u16::from(self.read(index));
    }

    fn read_word_as_u8(&self, index: u16, h: &mut u8, l: &mut u8) {
        *l = self.read(index);
        *h = self.read(index + 1);
    }

    fn write(&mut self, index: u16, val: u8) -> bool {
        if index < 0x2000 || index > 0x3fff {
            return false; //sollte false sein
        } else {
            self.memory[usize::from(index)] = val;
            return true;
        }
    }

    fn write_word(&mut self, index: u16, valH: u8, valL: u8) -> bool {
        return self.write(index, valL) & self.write(index + 1, valH);
    }

    fn write_word_with_u16(&mut self, index: u16, val: u16) -> bool {
        let le_bytes = val.to_le_bytes();
        return self.write_word(index, le_bytes[1], le_bytes[0]);
    }
}

pub struct Special {
    pub shift_offset: u8,
    pub shift0: u8,
    pub shift1: u8,
}

impl Special {
    fn machine_out(&mut self, port: u8, value: u8) {
        match port {
            2 => {
                self.shift_offset = value & 0x7;
            }
            4 => {
                self.shift0 = self.shift1;
                self.shift1 = value;
            }
            _ => {
                if port != 3 && port != 5 && port != 6 {
                    println!("unimplemented special port(out): {:?}", port);
                }
            }
        }
    }
    fn machine_in(&mut self, port: u8) -> u8 {
        let mut a: u8 = 0;
        match port {
            3 => {
                let v: u16 = (u16::from(self.shift1) << 8) | u16::from(self.shift0);
                let buffer: u16 = (v >> (8 - self.shift_offset)) & 0xff;
                a = buffer.to_be_bytes()[1];
            }
            _ => {
                if port != 1 && port != 2 {
                    println!("unimplemented special port(in): {:?}", port);
                }
            }
        }
        return a;
    }
}

const CYCLES: [u8; 256] = [
    4, 10, 7, 5, 5, 5, 7, 4, 4, 10, 7, 5, 5, 5, 7, 4, 4, 10, 7, 5, 5, 5, 7, 4, 4, 10, 7, 5, 5, 5,
    7, 4, 4, 10, 16, 5, 5, 5, 7, 4, 4, 10, 16, 5, 5, 5, 7, 4, 4, 10, 13, 5, 10, 10, 10, 4, 4, 10,
    13, 5, 5, 5, 7, 4, 5, 5, 5, 5, 5, 5, 7, 5, 5, 5, 5, 5, 5, 5, 7, 5, 5, 5, 5, 5, 5, 5, 7, 5, 5,
    5, 5, 5, 5, 5, 7, 5, 5, 5, 5, 5, 5, 5, 7, 5, 5, 5, 5, 5, 5, 5, 7, 5, 7, 7, 7, 7, 7, 7, 7, 7, 5,
    5, 5, 5, 5, 5, 7, 5, 4, 4, 4, 4, 4, 4, 7, 4, 4, 4, 4, 4, 4, 4, 7, 4, 4, 4, 4, 4, 4, 4, 7, 4, 4,
    4, 4, 4, 4, 4, 7, 4, 4, 4, 4, 4, 4, 4, 7, 4, 4, 4, 4, 4, 4, 4, 7, 4, 4, 4, 4, 4, 4, 4, 7, 4, 4,
    4, 4, 4, 4, 4, 7, 4, 11, 10, 10, 10, 17, 11, 7, 11, 11, 10, 10, 10, 17, 17, 7, 11, 11, 10, 10,
    10, 17, 11, 7, 11, 11, 10, 10, 10, 17, 17, 7, 11, 11, 10, 10, 18, 17, 11, 7, 11, 11, 5, 10, 5,
    17, 17, 7, 11, 11, 10, 10, 4, 17, 11, 7, 11, 11, 5, 10, 4, 17, 17, 7, 11,
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
