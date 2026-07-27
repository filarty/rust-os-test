use crate::font::{ASCII_FONT};
use core::fmt::{self, Write};


const FONT_WIDTH: usize = 8;
const FONT_HEIGHT: usize = 16;

const SCREEN_WIDTH: usize = 800;
const SCREEN_HEIGHT: usize = 1000;
const BUFFER_SIZE: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

static mut RAM_BUFFER: [u32; BUFFER_SIZE] = [0u32; BUFFER_SIZE];

pub struct Console {
    x: usize,
    y: usize,
    stride: usize,
    buff_addr: *mut u32,
}


impl Console {

    pub fn new(x: usize, y: usize, stride: usize, buff_addr: *mut u32) -> Self {
        Self {
            x,
            y,
            stride,
            buff_addr,
        }
    }
    pub fn flush(&mut self) {
         unsafe {
            let ptr = core::ptr::addr_of_mut!(RAM_BUFFER) as *const u32;
            core::ptr::copy_nonoverlapping(
                ptr, 
                self.buff_addr, 
                BUFFER_SIZE,
            );
        }
    }

    pub fn clean_screen(&mut self) {
        unsafe {
            for i in 0..BUFFER_SIZE {
                let ptr = core::ptr::addr_of_mut!(RAM_BUFFER) as *mut u32;
                ptr.add(i).write_volatile(0x00324FFF);
            }
        }
        self.flush();
    }

    fn draw_char(&mut self, c: char) {
        if c == '\n' {
            self.x = 0;
            self.y += 1;
            return;
        }
        let start_pixel_x = self.x * FONT_WIDTH;
        let start_pixel_y = self.y * FONT_HEIGHT;

        let glyph = ASCII_FONT[c as usize];
        for row in 0..FONT_HEIGHT {
            for col in 0..FONT_WIDTH {
                if glyph[row] & (1 << col) != 0 {
                    let px = start_pixel_x + col;
                    let py = start_pixel_y + row;

                    let byte_offset = (py * self.stride) + px;
                    unsafe {
                        let ptr = core::ptr::addr_of_mut!(RAM_BUFFER) as *mut u32;
                        ptr.add(byte_offset).write_volatile(0xFFFFFFFF);
                    }
                }
            }
        }
        self.x += 1;

    }
}

impl Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.draw_char(c);
        }
        unsafe {
            let ptr = core::ptr::addr_of_mut!(RAM_BUFFER) as *const u32;
            core::ptr::copy_nonoverlapping(
                ptr, 
                self.buff_addr, 
                BUFFER_SIZE,
            );
        }
        self.flush();
        Ok(())
    }
}


