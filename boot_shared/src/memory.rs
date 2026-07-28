use uefi::boot::MemoryType;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct BootMemRegion {
    pub base: u64,
    pub length: u64,
    pub kind: MemRegionKind,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum MemRegionKind {
    Usable = 0,
    Reserved = 1,
    AcpiReclaimble = 2,
    AcpiNvs = 3,
    BadMemory = 4,
    BootloaderReclaimble = 5,
    UefiRuntime = 6,
}


pub fn convert_type(ty: MemoryType) -> MemRegionKind {
    match ty {
        MemoryType::CONVENTIONAL
        | MemoryType::BOOT_SERVICES_CODE
        | MemoryType::BOOT_SERVICES_DATA => MemRegionKind::Usable,
        MemoryType::ACPI_RECLAIM => MemRegionKind::AcpiReclaimble,
        MemoryType::ACPI_NON_VOLATILE => MemRegionKind::AcpiNvs,
        MemoryType::UNUSABLE => MemRegionKind::BadMemory,
        MemoryType::LOADER_CODE
        | MemoryType::LOADER_DATA => MemRegionKind::BootloaderReclaimble,
        MemoryType::RUNTIME_SERVICES_CODE
        | MemoryType::RUNTIME_SERVICES_DATA => MemRegionKind::UefiRuntime,
        _ => MemRegionKind::Reserved,
    }
}