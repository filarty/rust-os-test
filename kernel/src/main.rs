#![no_std]
#![no_main]

use core::{fmt::Write, panic::PanicInfo};
use boot_shared::BootInfo;
use kernel::console::Console;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}


#[unsafe(no_mangle)]
pub extern "sysv64" fn _start(boot_info: *const BootInfo) -> ! {
    
    unsafe { core::arch::asm!("cli"); }

    unsafe {
        let info: BootInfo = *boot_info;

        let buff = info.video_buff;
        let buff_addr = buff.addr as *mut u32;
        
        let mut console = Console::new(
            0,
            0,
            buff.stride as usize,
            buff_addr,
        );

        console.clean_screen();
        

        let _ = console.write_str("Hello, world!\n");

        let _ = console.write_str("Test OS Check...\n");

        loop {
            core::arch::asm!("hlt");
        }
    }
}