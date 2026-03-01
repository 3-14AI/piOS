#![allow(unused_imports)]
use vstd::prelude::*;

#[cfg(feature = "verus")]
verus! {

pub const PAGE_SIZE: u64 = 4096;
pub const BLOCK_SIZE: u64 = 32;
pub const BLOCKS_PER_PAGE: usize = 128; // 4096 / 32

pub struct BitmapSlabAllocator {
    pub page_addr: u64,
    pub bitmap: [u64; 2], // 128 bits. 0 = free, 1 = allocated.
}

impl BitmapSlabAllocator {
    pub open spec fn valid(&self) -> bool {
        self.page_addr % PAGE_SIZE == 0
    }

    pub fn new(page_addr: u64) -> (s: Self)
        requires page_addr % PAGE_SIZE == 0
        ensures s.valid()
    {
        BitmapSlabAllocator {
            page_addr,
            bitmap: [0, 0],
        }
    }

    pub fn alloc(&mut self) -> (res: Option<u64>)
        requires old(self).valid()
        ensures
            self.valid(),
            match res {
                Some(addr) => {
                    // Result address is within the page
                    addr >= self.page_addr && addr < self.page_addr + PAGE_SIZE &&
                    // And aligned
                    addr % BLOCK_SIZE == 0
                },
                None => true,
            }
    {
        // Check first word
        let mut word_idx = 0;
        while word_idx < 2
            invariant
                0 <= word_idx <= 2,
                self.valid(),
            decreases
                2 - word_idx,
        {
            let word = self.bitmap[word_idx];
            if word != 0xFFFF_FFFF_FFFF_FFFFu64 {
                // Found a word with a free bit
                let mut bit_idx = 0;
                while bit_idx < 64
                    invariant
                        0 <= bit_idx <= 64,
                        0 <= word_idx < 2,
                        self.valid(),
                    decreases
                        64 - bit_idx,
                {
                    let mask = 1u64 << bit_idx;
                    if (word & mask) == 0 {
                        // Found free bit
                        self.bitmap[word_idx] = word | mask;
                        let offset = (word_idx as u64 * 64 + bit_idx as u64) * BLOCK_SIZE;
                        return Some(self.page_addr + offset);
                    }
                    bit_idx = bit_idx + 1;
                }
            }
            word_idx = word_idx + 1;
        }
        None
    }

    pub fn dealloc(&mut self, ptr: u64)
        requires
            old(self).valid(),
            // Ptr must be within this page
            ptr >= old(self).page_addr,
            ptr < old(self).page_addr + PAGE_SIZE,
            ((ptr - old(self).page_addr) as int) % (BLOCK_SIZE as int) == 0,
        ensures
            self.valid(),
    {
        let offset = ptr - self.page_addr;
        let block_idx = offset / BLOCK_SIZE;
        let word_idx = (block_idx / 64) as usize;
        let bit_idx = (block_idx % 64) as u64;

        if word_idx < 2 {
            let mask = 1u64 << bit_idx;
            self.bitmap[word_idx] = self.bitmap[word_idx] & !mask;
        }
    }
}

} // verus!

#[cfg(feature = "verus")]
use core::alloc::{GlobalAlloc, Layout};

#[cfg(feature = "verus")]
pub struct KernelAllocator;

// Global static for the allocator state
#[cfg(feature = "verus")]
static mut SLAB: Option<BitmapSlabAllocator> = None;

#[cfg(feature = "verus")]
unsafe impl GlobalAlloc for KernelAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // NOTE: This is a simplified single-threaded implementation.
        // It uses unsafe access to static muts.
        // Synchronization should be added with verified spinlocks (WP-006).

        // We only support allocations <= 32 bytes for this verified slab
        if layout.size() > 32 {
            return core::ptr::null_mut();
        }

        // Initialize Global PMM if needed (lazy init)
        if crate::pmm::GLOBAL_PMM.is_none() {
            crate::pmm::GLOBAL_PMM = Some(crate::pmm::PhysicalMemoryManager::new());
        }

        // Initialize Slab if needed
        if SLAB.is_none() {
            if let Some(pmm) = &mut crate::pmm::GLOBAL_PMM {
                if let Some(page_addr) = pmm.alloc() {
                    SLAB = Some(BitmapSlabAllocator::new(page_addr));
                } else {
                    return core::ptr::null_mut();
                }
            } else {
                return core::ptr::null_mut();
            }
        }

        if let Some(slab) = &mut SLAB {
            if let Some(addr) = slab.alloc() {
                return addr as *mut u8;
            }
        }

        core::ptr::null_mut()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if let Some(slab) = &mut SLAB {
            let addr = ptr as u64;
            if addr >= slab.page_addr && addr < slab.page_addr + 4096 {
                slab.dealloc(addr);
            }
        }
    }
}

#[cfg(feature = "verus")]
unsafe impl Sync for KernelAllocator {}

#[cfg(feature = "verus")]
#[global_allocator]
static ALLOCATOR: KernelAllocator = KernelAllocator;
