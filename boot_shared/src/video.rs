use crate::memory::BootMemRegion;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum PixelFormat {
    Rgb = 0,
    Bgr = 1,
    Bitmask = 2,
    BltOnly = 3,
}


#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VideoBuffer {
    pub addr: u64,
    pub size: u64,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub pixel_format: PixelFormat,
}


#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BootInfo {
    pub video_buff: VideoBuffer,
    pub memory_map_ptr: *const BootMemRegion,
    pub memory_map_len: usize,
}

