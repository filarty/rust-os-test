#![no_std]
#![no_main]

use core::panic::PanicInfo;
use boot_shared::BootInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

fn delay(count: u64) {
    for _ in 0..count {
        core::hint::spin_loop();
    }
}

#[unsafe(no_mangle)]
pub extern "sysv64" fn _start(boot_info: *const BootInfo) -> ! {
    
    unsafe { core::arch::asm!("cli"); }

    unsafe {
        let info: BootInfo = *boot_info;

        let buff = info.video_buff;
        let buff_addr = buff.addr as *mut u32;
        let total_pixels = (buff.height * buff.stride) as usize;

        for i in 0..total_pixels {
            buff_addr.add(i).write_volatile(0xFFFFFF);
        }

        for i in 0..total_pixels {
            buff_addr.add(i).write_volatile(0xF3BA35);
            delay(2_000_000);
        }

    }
    loop {}
}