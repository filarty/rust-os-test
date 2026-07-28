#![no_main]
#![no_std]

use core::mem::transmute;
use core::slice;
use log::info;
use uefi::boot::{self, MemoryType, SearchType};
use uefi::mem::memory_map::MemoryMap;
use uefi::proto::media::file::{File, FileAttribute};
use uefi::proto::media::file::{FileMode, FileInfo};
use boot_shared::{ BootInfo, VideoBuffer, PixelFormat };
use uefi::proto::console::gop::{ GraphicsOutput };
use uefi::runtime::{self};
use uefi::{cstr16};
use uefi::prelude::*;
use uefi::proto::device_path::text::{
    AllowShortcuts, DevicePathToText, DisplayOnly, 
};
use uefi::proto::loaded_image::{LoadedImage};
use uefi::{Identify, Result};
use boot_shared::memory::{MemRegionKind, BootMemRegion, convert_type};
use xmas_elf::program::Type;

static mut BOOT_INFO: Option<BootInfo> = None;
static mut REGION_BUFF: [BootMemRegion; 256] = [
    BootMemRegion {
        base: 0,
        length: 0,
        kind: MemRegionKind::Reserved,
    }; 256];

fn print_image_path() -> Result {
    let loaded_image = 
    boot::open_protocol_exclusive::<LoadedImage>(boot::image_handle())?;

    let device_path_to_text_handle = *boot::locate_handle_buffer(
        SearchType::ByProtocol(&DevicePathToText::GUID),
    )?
    .first()
    .expect("DevicePathToText is missing");

    let device_path_to_text = 
    boot::open_protocol_exclusive::<DevicePathToText>(device_path_to_text_handle, )?;


    let image_device_path = 
        loaded_image.file_path().expect("File path is not set");
    let image_device_path_text = device_path_to_text
        .convert_device_path_to_text(
            image_device_path,
            DisplayOnly(true),
            AllowShortcuts(false),
        )
        .expect("convert_device_path_to_text failed");

    info!("Image path: {}", &*image_device_path_text);
    Ok(())
}


fn print_image_size() -> Result {
    
    let loaded_image = 
        boot ::open_protocol_exclusive::<LoadedImage>(boot::image_handle());
    
    let (_, image_size) = loaded_image
        .expect("Error with image!")
        .info();
    
    info!("Image Size: {} KiB", &image_size / 1024);

    Ok(())
}


fn print_time_stamp() -> Result {
    let timestamp_service = runtime::get_time().expect("Error with time!");
    info!("Start time: {}:{}:{} ", &timestamp_service.hour(), &timestamp_service.minute(), &timestamp_service.second());
    Ok(())
}


fn load_kernel_on_memory() -> Result<u64> {
    let mut file_system_image = boot::get_image_file_system(boot::image_handle())?;
    let mut dir = file_system_image.open_volume()?;
    let file = dir.open(cstr16!("boot\\kernel.bin"), FileMode::Read, FileAttribute::READ_ONLY)?;
    let mut regular_file = file.into_regular_file().ok_or(Status::UNSUPPORTED)?;

    let mut file_info_buff = [0u8; 256];
    let info = regular_file.get_info::<FileInfo>(&mut file_info_buff).unwrap();
    let kernel_size = info.file_size() as usize;

    let temp_pages = (kernel_size + 4095) / boot::PAGE_SIZE;
    let temp_ptr = boot::allocate_pages(boot::AllocateType::AnyPages, boot::MemoryType::LOADER_DATA, temp_pages)?;
    let temp_buffer = unsafe { slice::from_raw_parts_mut(temp_ptr.as_ptr(), kernel_size) };
    regular_file.read(temp_buffer)?;

    let elf = xmas_elf::ElfFile::new(temp_buffer).expect("Failed to parse ELF");

    let mut min_vaddr = usize::MAX;
    let mut max_vaddr = 0;

    for ph in elf.program_iter() {
        if ph.get_type().unwrap() == Type::Load {
            let vaddr = ph.virtual_addr() as usize;
            let mem_size = ph.mem_size() as usize;
            if mem_size == 0 { continue; }

            if vaddr < min_vaddr { min_vaddr = vaddr; }
            if vaddr + mem_size > max_vaddr { max_vaddr = vaddr + mem_size; }
        }
    }

    let alloc_start = min_vaddr & !0xFFF;
    let alloc_end = (max_vaddr + 0xFFF) & !0xFFF;
    let total_pages = (alloc_end - alloc_start) / boot::PAGE_SIZE;

    info!("Allocating {} pages for entire kernel from {:#x} to {:#x}", total_pages, alloc_start, alloc_end);

    boot::allocate_pages(
        boot::AllocateType::Address(alloc_start as u64),
        boot::MemoryType::LOADER_DATA,
        total_pages,
    ).expect("Failed to allocate memory for the kernel!");


    for ph in elf.program_iter() {
        if ph.get_type().unwrap() == Type::Load {
            let vaddr = ph.virtual_addr() as usize;
            let mem_size = ph.mem_size() as usize;
            let file_size = ph.file_size() as usize;
            let offset = ph.offset() as usize;

            if mem_size == 0 { continue; }

            info!("Loading segment to {:#x}, size: {} bytes", vaddr, mem_size);

            let segment_ptr = vaddr as *mut u8;
            let segment_mem = unsafe { slice::from_raw_parts_mut(segment_ptr, mem_size) };

            let file_data = &temp_buffer[offset..(offset + file_size)];
            segment_mem[..file_size].copy_from_slice(file_data);

            if mem_size > file_size {
                segment_mem[file_size..].fill(0);
            }
        }
    }

    Ok(elf.header.pt2.entry_point())
}

fn jmp_in_kernel(addr_ptr: *const (), boot_info: *const BootInfo) {
    let kernel_main: extern "sysv64" fn(boot_info: *const BootInfo) -> ! = unsafe {
        transmute(addr_ptr)
    };
    kernel_main(boot_info)
}

fn prepare_video_buffer() -> Result<VideoBuffer> {
    
    let graphic_hander = boot::get_handle_for_protocol::<GraphicsOutput>()?;
    let mut graphics_output_proto = boot::open_protocol_exclusive::<GraphicsOutput>(graphic_hander)?;

    let mode_info = graphics_output_proto.current_mode_info();
    let mut framebuffer = graphics_output_proto.frame_buffer();

    let video_buff =  VideoBuffer {
        addr: framebuffer.as_mut_ptr() as u64,
        size: framebuffer.size() as u64,
        width: mode_info.resolution().0 as u32,
        height: mode_info.resolution().1 as u32,
        stride: mode_info.stride() as u32,
        pixel_format: PixelFormat::Rgb,
    };
    
    Ok(video_buff)
}

fn prepare_memory_map() -> Result<usize> {
    let memory_map = boot::memory_map(MemoryType::LOADER_DATA).expect("Error with memory map");
    let mut count = 0usize;
    for desc in memory_map.entries() {
        let ln = desc.page_count * 4096;
        unsafe { REGION_BUFF[count] = BootMemRegion { 
            base: desc.phys_start, 
            length: ln, 
            kind: convert_type(desc.ty)
        };
        info!{"Memory for {:#?} in range {}:{} is reserved", desc.ty, desc.phys_start, ln}
    }
        count += 1;
    }
    Ok(count)
}

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    print_time_stamp().unwrap();
    print_image_path().unwrap();
    print_image_size().unwrap();

    let entry_point = load_kernel_on_memory().unwrap();

    info!("Kernel is successfully loaded in memory!");
    info!("Jumping to entry point: {:#x}", entry_point);
    info!("Goodbye from bootloader!");

    let size = prepare_memory_map().expect("Error in prepare mem");


    unsafe {
        BOOT_INFO = Some(
            BootInfo { 
                video_buff: prepare_video_buffer().expect("Error with video!"), 
                memory_map_ptr: core::ptr::addr_of!(REGION_BUFF) as *const BootMemRegion,
                memory_map_len: size,
            }
        )
    }

    let boot_info_ptr = unsafe {

        let opt_ptr = core::ptr::addr_of!(BOOT_INFO);

        (*opt_ptr).as_ref().unwrap() as *const BootInfo
    };

    unsafe { let _ = boot::exit_boot_services(None); };

    jmp_in_kernel(entry_point as *const (), boot_info_ptr);   
    
    Status::SUCCESS
}




