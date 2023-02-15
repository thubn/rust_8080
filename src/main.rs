//#[allow(dead_code)]
//mod render;
mod cpu;

//use std::io;

use cpu::state8080::ConditionCodes;
use cpu::state8080::Memory;
use cpu::state8080::Special;
use cpu::state8080::State8080;

use minifb::{Window, /*ScaleMode,*/ WindowOptions};

const SCREEN_WIDTH: usize = 224;
const SCREEN_HEIGHT: usize = 256;
const NUM_PIXELS: usize = SCREEN_HEIGHT * SCREEN_WIDTH;

fn main() {
    //let memor: Memory = Memory { memory: () }
    let mut cpu: State8080 = State8080 {
        a: 0,
        b: 0,
        c: 0,
        d: 0,
        e: 0,
        h: 0,
        l: 0,
        sp: 0,
        pc: 0,
        memory: Memory { memory: [0; 16384] },
        cc: ConditionCodes {
            s: false,
            z: false,
            five: false,
            ac: false,
            three: false,
            p: false,
            one: true,
            cy: false,
        },
        int_enable: false,
        int_type: false,
        int_vector: 0,
        int_delay: 0,
        cycles: 0,
        special: Special {
            shift_offset: 0,
            shift0: 0,
            shift1: 0,
        },
    };

    cpu.load_rom(0..=0x07ff, "invaders.h");
    cpu.load_rom(0x0800..=0x0fff, "invaders.g");
    cpu.load_rom(0x1000..=0x17ff, "invaders.f");
    cpu.load_rom(0x1800..=0x1fff, "invaders.e");

    // minifb window
    let mut window = Window::new(
        "rust_8080",
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });
    window.limit_update_rate(Some(std::time::Duration::from_millis(2)));

    println!("Created Window.. Entering mainloop..");

    loop {
        while !cpu.step() {
            /*if cpu.get_hl() > 0x4000 {
                println!("PC is at {:x?}. HL: {:x?} Press enter to continue", cpu.pc, cpu.get_hl());
                let mut buffer = String::new();
                io::stdin()
                    .read_line(&mut buffer)
                    .expect("Did not enter a correct string");
                if buffer.contains("mem") {
                    cpu.dump_memory();
                }
            }*/
        }

        // render pixels from emulated ram in minifb window
        let mut buffer: Vec<u32> = vec![0; NUM_PIXELS];
        let mut j = 0;
        for row in (0x2400..=0x241f).rev() {
            for b in (0..=7).rev() {
                for col in 0..224 {
                    let offset = row + (col * 0x20);
                    if (cpu.memory.memory[offset] & (0x1 << b)) != 0x0 {
                        buffer[j] = 0x00ffffff;
                    } else {
                        buffer[j] = 0x00000000;
                    }
                    j += 1;
                }
            }
        }

        window
            .update_with_buffer(&buffer, SCREEN_WIDTH, SCREEN_HEIGHT)
            .unwrap_or_else(|e| {
                panic!("{}", e);
            });
    }
}
