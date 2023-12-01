use core::alloc::Layout;
use core::ptr::NonNull;

use crate::{AllocError, AllocResult, BaseAllocator, ByteAllocator, PageAllocator};

pub struct EarlyAllocator<const PAGE_SIZE: usize> {
    start: usize,
    end: usize,
    next: usize,
    used_bytes: usize,
    used_pages: usize,
}

impl<const PAGE_SIZE: usize> EarlyAllocator<PAGE_SIZE> {
    pub const fn new() -> Self {
        Self {
            start: 0,
            end: 0,
            next: 0,
            used_bytes: 0,
            used_pages: 0,
        }
    }
}

impl<const PAGE_SIZE: usize> BaseAllocator for EarlyAllocator<PAGE_SIZE> {
    fn init(&mut self, start_vaddr: usize, size: usize) {
        let end = start_vaddr + size;
        self.start = start_vaddr;
        self.end = end;
        self.next = start_vaddr;
        self.used_bytes = 0;
        self.used_pages = 0;
    }

    fn add_memory(&mut self, _start_vaddr: usize, _size: usize) -> AllocResult {
        Ok(())
    }
}

impl<const PAGE_SIZE: usize> ByteAllocator for EarlyAllocator<PAGE_SIZE> {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        let size = self.calculate_alloc_size(layout);

        let aligned_addr = self.align_up(self.next, layout.align());

        if aligned_addr + size <= self.end {
            self.next = aligned_addr + size;
            self.used_bytes += size;
            Ok(NonNull::new(aligned_addr as *mut u8).unwrap())
        } else {
            Err(AllocError::NoMemory)
        }
    }

    fn dealloc(&mut self, _ptr: NonNull<u8>, layout: Layout) {
        self.used_bytes -= layout.size();
        if self.used_bytes == 0 {
            self.next = self.start;
        }
    }

    fn total_bytes(&self) -> usize {
        self.end - self.start
    }

    fn used_bytes(&self) -> usize {
        self.used_bytes
    }

    fn available_bytes(&self) -> usize {
        self.end - self.next
    }
}

impl<const PAGE_SIZE: usize> PageAllocator for EarlyAllocator<PAGE_SIZE> {
    const PAGE_SIZE: usize = PAGE_SIZE;

    fn alloc_pages(&mut self, num_pages: usize, align_pow2: usize) -> AllocResult<usize> {
        self.validate_page_alloc_params(align_pow2)?;

        let size = num_pages * PAGE_SIZE;
        if self.next + size <= self.end {
            let aligned_addr = self.align_up(self.next, align_pow2);
            self.next = aligned_addr + size;
            self.used_pages += num_pages;
            Ok(aligned_addr)
        } else {
            Err(AllocError::NoMemory)
        }
    }

    fn dealloc_pages(&mut self, _pos: usize, _num_pages: usize) {
        unimplemented!()
    }

    fn total_pages(&self) -> usize {
        self.calculate_pages(self.end - self.start)
    }

    fn used_pages(&self) -> usize {
        self.used_pages
    }

    fn available_pages(&self) -> usize {
        self.calculate_pages(self.end - self.next)
    }
}

impl<const PAGE_SIZE: usize> EarlyAllocator<PAGE_SIZE> {
    fn validate_page_alloc_params(&self, align_pow2: usize) -> AllocResult<()> {
        if align_pow2 % PAGE_SIZE != 0 {
            return Err(AllocError::InvalidParam);
        }

        let align_pow2 = align_pow2 / PAGE_SIZE;
        if !align_pow2.is_power_of_two() {
            return Err(AllocError::InvalidParam);
        }

        Ok(())
    }

    fn calculate_pages(&self, bytes: usize) -> usize {
        bytes / PAGE_SIZE
    }

    fn calculate_alloc_size(&self, layout: Layout) -> usize {
        layout.size().next_power_of_two().max(
            layout.align().max(
                layout.align().max(core::mem::size_of::<usize>())
            )
        )
    }

    fn align_up(&self, addr: usize, align: usize) -> usize {
        (addr + align - 1) & !(align - 1)
    }
}
