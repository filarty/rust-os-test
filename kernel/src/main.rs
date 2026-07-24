#![no_std]
#![no_main]

mod print;

use core::panic::PanicInfo;
use boot_shared::BootInfo;

use crate::print::ASCII_FONT;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

static mut Y_SCREEN: usize = 0;

#[unsafe(no_mangle)]
pub extern "sysv64" fn _start(boot_info: *const BootInfo) -> ! {
    
    unsafe { core::arch::asm!("cli"); }

    unsafe {
        let info: BootInfo = *boot_info;

        let buff = info.video_buff;
        let buff_addr = buff.addr as *mut u32;
        let total_pixels = (buff.height * buff.stride) as usize;


    fn print_gliph(
        ch: char, 
        stride: usize, 
        x: usize, 
        y: usize, 
        buff_addr: *mut u32, 
        color: u32) {
    
        let glyph = ASCII_FONT[ch as usize];

        for row in 0..16 {
            let line = glyph[row];
            for col in 0..8 {
                let x_screen = x + col;
                let y_screen = y + row;
                let index = y_screen * stride + x_screen;
                unsafe {
                if ( line & (1 << col)) != 0 {
                    buff_addr.add(index).write_volatile(color);
                    }
                }
            }
        }
        
    }

    fn print_string(value: &str, buff_addr: *mut u32, stride: usize) {
        let mut x = 50;
        for i in value.chars() {
            unsafe {
            print_gliph(i, stride as usize, x, Y_SCREEN, buff_addr, 0x0000000);}
            x+=8
        }
    }

    fn print(s: &str, buff_addr: *mut u32, stride: usize) {
        print_string(s, buff_addr, stride);
        unsafe {
            Y_SCREEN += 20;
        }
    }

        for i in 0..total_pixels {
            buff_addr.add(i).write_volatile(0xFFFFFF);
        }

        print("Hello, world", buff_addr, buff.stride as usize);
        print("User", buff_addr, buff.stride as usize);
        print("123456789", buff_addr, buff.stride as usize);
        print("!##@$!%#^!", buff_addr, buff.stride as usize);
    }
    loop {}
}