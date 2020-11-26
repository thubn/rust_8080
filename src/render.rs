const SCREEN_WIDTH: usize = 224;
const SCREEN_HEIGHT: usize = 256;
const NUM_PIXELS: usize = SCREEN_HEIGHT * SCREEN_WIDTH;

pub fn render_frame(memory: &[u8], mut window: minifb::Window) -> minifb::Window{
    let mut buffer: Vec<u32> = Vec::with_capacity(NUM_PIXELS + 1);
    for (i, item) in memory.iter().enumerate() {
        for shift in 7..=0 {
            let pixel = ((*item >> shift) & 1) == 1;
            if pixel {
                buffer[i*8+shift] = 0x00ffffff;
            } else {
                buffer[i*8+shift] = 0x0;
            }
        }
    }
    window.update_with_buffer(&buffer, SCREEN_WIDTH, SCREEN_HEIGHT).unwrap();
    return window;
}