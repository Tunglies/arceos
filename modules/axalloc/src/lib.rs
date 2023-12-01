//! [ArceOS](https://github.com/rcore-os/arceos) global memory allocator.
//!
//! It provides [`GlobalAllocator`], which implements the trait
//! [`core::alloc::GlobalAlloc`]. A static global variable of type
//! [`GlobalAllocator`] is defined with the `#[global_allocator]` attribute, to
//! be registered as the standard library’s default allocator.

#![no_std]

#[macro_use]
extern crate log;
extern crate alloc;

mod page;

use allocator::{AllocResult, BaseAllocator, ByteAllocator, PageAllocator};
use core::alloc::{GlobalAlloc, Layout};
use core::ptr::NonNull;
use spinlock::SpinNoIrq;

const PAGE_SIZE: usize = 0x1000;

pub use page::GlobalPage;

use allocator::EarlyAllocator as DefaultByteAllocator;

pub struct GlobalAllocator {
    inner: SpinNoIrq<DefaultByteAllocator<PAGE_SIZE>>,
}

impl GlobalAllocator {
    pub const fn new() -> Self {
        Self {
            inner: SpinNoIrq::new(DefaultByteAllocator::new()),
        }
    }

    pub const fn name(&self) -> &'static str {
        "early"
    }

    pub fn init(&self, start_vaddr: usize, size: usize) {
        self.inner.lock().init(start_vaddr, size);
    }

    pub fn add_memory(&self, _start_vaddr: usize, _size: usize) -> AllocResult {
        unimplemented!()
    }

    pub fn alloc(&self, layout: Layout) -> AllocResult<NonNull<u8>> {
        self.inner.lock().alloc(layout)
    }

    
    pub fn dealloc(&self, pos: NonNull<u8>, layout: Layout) {
        self.inner.lock().dealloc(pos, layout)
    }

    pub fn alloc_pages(&self, num_pages: usize, align_pow2: usize) -> AllocResult<usize> {
        self.inner.lock().alloc_pages(num_pages, align_pow2)
    }

    pub fn dealloc_pages(&self, pos: usize, num_pages: usize) {
        self.inner.lock().dealloc_pages(pos, num_pages)
    }

    pub fn used_bytes(&self) -> usize {
        self.inner.lock().used_bytes()
    }

    pub fn available_bytes(&self) -> usize {
        self.inner.lock().available_bytes()
    }

    pub fn used_pages(&self) -> usize {
        self.inner.lock().used_pages()
    }

    pub fn available_pages(&self) -> usize {
        self.inner.lock().available_pages()
    }
}

unsafe impl GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if let Ok(ptr) = GlobalAllocator::alloc(self, layout) {
            ptr.as_ptr()
        } else {
            alloc::alloc::handle_alloc_error(layout)
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        GlobalAllocator::dealloc(self, NonNull::new(ptr).expect("dealloc null ptr"), layout)
    }
}

#[cfg_attr(all(target_os = "none", not(test)), global_allocator)]
static GLOBAL_ALLOCATOR: GlobalAllocator = GlobalAllocator::new();

/// Returns the reference to the global allocator.
pub fn global_allocator() -> &'static GlobalAllocator {
    &GLOBAL_ALLOCATOR
}

/// Initializes the global allocator with the given memory region.
///
/// Note that the memory region bounds are just numbers, and the allocator
/// does not actually access the region. Users should ensure that the region
/// is valid and not being used by others, so that the allocated memory is also
/// valid.
///
/// This function should be called only once, and before any allocation.
pub fn global_init(start_vaddr: usize, size: usize) {
    debug!(
        "initialize global allocator at: [{:#x}, {:#x})",
        start_vaddr,
        start_vaddr + size
    );
    GLOBAL_ALLOCATOR.init(start_vaddr, size);
}

/// Add the given memory region to the global allocator.
///
/// Users should ensure that the region is valid and not being used by others,
/// so that the allocated memory is also valid.
///
/// It's similar to [`global_init`], but can be called multiple times.
pub fn global_add_memory(start_vaddr: usize, size: usize) -> AllocResult {
    debug!(
        "add a memory region to global allocator: [{:#x}, {:#x})",
        start_vaddr,
        start_vaddr + size
    );
    GLOBAL_ALLOCATOR.add_memory(start_vaddr, size)
}
