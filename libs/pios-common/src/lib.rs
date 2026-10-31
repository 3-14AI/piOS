#![no_std]

#[cfg(test)]
extern crate std;

use linked_list_allocator::LockedHeap;

// We do NOT use #[global_allocator] here so consumer crates can do it.
// We just expose a shared heap instance and the DummyAllocator wrapper.

pub static HEAP: LockedHeap = LockedHeap::empty();

pub fn common_init() -> bool {
    true
}

// In standard library mode (tests), we can just wrap the system allocator
#[cfg(test)]
pub struct DummyAllocator;
#[cfg(test)]
unsafe impl core::alloc::GlobalAlloc for DummyAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        std::alloc::alloc(layout)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        std::alloc::dealloc(ptr, layout)
    }
}

// In no_std mode (bare metal), we forward to our LockedHeap
#[cfg(not(test))]
pub struct DummyAllocator;
#[cfg(not(test))]
unsafe impl core::alloc::GlobalAlloc for DummyAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        HEAP.alloc(layout)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        HEAP.dealloc(ptr, layout)
    }
}

/// # Safety
///
/// The caller must ensure that the `heap_start` pointer and `heap_size`
/// correctly describe a valid, unaliased region of memory that is safe
/// to use as a heap. This region must not overlap with other active memory
/// and must be mapped correctly.
pub unsafe fn init_allocator(heap_start: *mut u8, heap_size: usize) {
    HEAP.lock().init(heap_start, heap_size);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common_init() {
        assert!(common_init());
    }
}
