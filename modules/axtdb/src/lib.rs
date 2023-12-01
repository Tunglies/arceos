#![no_std]

extern crate fdt;
extern crate alloc;

use alloc::vec::Vec;
use core::convert::TryInto;

pub struct DtbInfo {
    pub memory_addr: usize,
    pub memory_size: usize,
    pub mmio_regions: Vec<(usize, usize)>,
}

impl DtbInfo {
    fn new(memory_addr: usize, memory_size: usize, mmio_regions: Vec<(usize, usize)>) -> Self {
        DtbInfo {
            memory_addr,
            memory_size,
            mmio_regions,
        }
    }
}

pub fn parse_dtb(dtb_pa: usize) -> Result<DtbInfo, &'static str> {
    let fdt = unsafe { fdt::Fdt::from_ptr(dtb_pa as *const u8) }
        .map_err(|_| "Failed to create Fdt instance")?;

    let memory_info = parse_memory_info(&fdt)?;
    let mmio_regions = parse_mmio_regions(&fdt)?;

    Ok(DtbInfo::new(memory_info.0, memory_info.1, mmio_regions))
}

fn parse_memory_info(fdt: &fdt::Fdt) -> Result<(usize, usize), &'static str> {
    let memory_node = fdt.find_node("/memory").ok_or("Memory node not found")?;
    let memory_region = memory_node.property("reg").ok_or("Memory region not found")?.value;
    let memory_addr: usize = match (&memory_region[0..8]).try_into() {
        Ok(bytes) => u64::from_be_bytes(bytes) as usize,
        Err(_) => return Err("Failed to convert memory address bytes"),
    };
    let memory_size: usize = match (&memory_region[8..16]).try_into() {
        Ok(bytes) => u64::from_be_bytes(bytes) as usize,
        Err(_) => return Err("Failed to convert memory address bytes"),
    };
    Ok((memory_addr, memory_size))
}

fn parse_mmio_regions(fdt: &fdt::Fdt) -> Result<Vec<(usize, usize)>, &'static str> {
    let mut mmio_regions = Vec::new();
    for node in fdt.find_all_nodes("/soc/virtio_mmio") {
        let virtio_mmio_range = node.property("reg").ok_or("Virtio_mmio region not found")?.value;
        let io_start = u64::from_be_bytes((&virtio_mmio_range[0..8]).try_into().unwrap()) as usize;
        let io_size = u64::from_be_bytes((&virtio_mmio_range[8..16]).try_into().unwrap()) as usize;
        mmio_regions.push((io_start, io_size));
    }

    Ok(mmio_regions)
}
