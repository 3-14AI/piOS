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
impl BitmapSlabAllocator {
    /// Analyzes internal fragmentation of the slab.
    /// Returns a score between 0.0 (perfectly packed or completely empty)
    /// and 1.0 (highly scattered allocations).
    pub fn analyze_fragmentation(&self) -> f32 {
        let mut total_allocated = 0;
        let mut gaps = 0;
        let mut in_gap = false;

        for word_idx in 0..2 {
            let word = self.bitmap[word_idx];
            for bit_idx in 0..64 {
                let mask = 1u64 << bit_idx;
                if (word & mask) != 0 {
                    total_allocated += 1;
                    if in_gap {
                        gaps += 1;
                        in_gap = false;
                    }
                } else {
                    if total_allocated > 0 {
                        in_gap = true;
                    }
                }
            }
        }

        if total_allocated == 0 || total_allocated == BLOCKS_PER_PAGE {
            return 0.0;
        }

        (gaps as f32) / (total_allocated as f32)
    }
}

#[cfg(feature = "verus")]
use alloc::vec;
#[cfg(feature = "verus")]
use core::alloc::{GlobalAlloc, Layout};
#[cfg(feature = "verus")]
use inference_runtime::{InferenceEngine, Tensor};

#[cfg(feature = "verus")]
pub struct DefragManager {
    engine: InferenceEngine,
    model_id: Option<usize>,
    context_id: Option<usize>,
}

#[cfg(feature = "verus")]
impl DefragManager {
    pub fn new() -> Self {
        let mut engine = InferenceEngine::new();
        // Load a mock predictive model for defragmentation strategy
        let model = engine.load_model_by_name("defrag_model").ok();
        let mut model_id = None;
        let mut context_id = None;

        if let Some(m) = model {
            model_id = Some(m.id);
            if let Ok(ctx) = engine.init_execution_context(&m) {
                context_id = Some(ctx);
            }
        }

        Self {
            engine,
            model_id,
            context_id,
        }
    }

    pub fn should_defrag(
        &mut self,
        pmm_fragmentation_score: f32,
        slab_fragmentation_score: f32,
    ) -> bool {
        // We use a fixed-size array to avoid heap allocations inside the allocator itself
        let mut input_data = [0u8; 4];
        input_data[0] = pmm_fragmentation_score.to_bits() as u8;
        input_data[1] = (pmm_fragmentation_score.to_bits() >> 8) as u8;
        input_data[2] = slab_fragmentation_score.to_bits() as u8;
        input_data[3] = (slab_fragmentation_score.to_bits() >> 8) as u8;

        if let Some(ctx) = self.context_id {
            // Create a tensor representing current fragmentation state without alloc
            // By wrapping it in a slice (we use a Vec internally in Tensor, but we can't avoid it without patching Tensor.
            // In a true no_std bare metal with this mock, we accept this simplification for the mock, but to be safe:
            // Since Tensor requires a Vec, we just use the fallback heuristic here to avoid re-entrant alloc panics
            // return (pmm_fragmentation_score + slab_fragmentation_score) > 1.5;

            // For the sake of the mock using InferenceEngine, we will create the Tensor but know it allocates.
            // To prevent recursion, we shouldn't call this inside `alloc`.

            let mut vec_data = alloc::vec::Vec::new();
            let _ = vec_data.try_reserve(4);
            vec_data.extend_from_slice(&input_data);

            let mut dim_data = alloc::vec::Vec::new();
            let _ = dim_data.try_reserve(2);
            dim_data.push(1);
            dim_data.push(4);

            let tensor = Tensor::new(vec_data, dim_data);

            if self.engine.set_input(ctx, 0, &tensor).is_ok() {
                if self.engine.compute(ctx).is_ok() {
                    let mut out = [0u8; 1];
                    if self.engine.get_output(ctx, 0, &mut out).is_ok() {
                        // In our mock, if output has high byte value, trigger defrag
                        return out[0] > 128;
                    }
                }
            }
        }

        // Fallback heuristic if AI model fails
        (pmm_fragmentation_score + slab_fragmentation_score) > 1.5
    }
}

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

        // Defragmentation hook check.
        // We periodically check and log defrag state using a simple counter simulation.
        // In a true environment, this would run in a separate kernel thread / optimizer daemon.
        if let Some(pmm) = &mut crate::pmm::GLOBAL_PMM {
            if pmm.defragment() {
                // High fragmentation detected.
                // An intent could be triggered here to ask userland 'sys_optimizer' to squeeze memory.
            }
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
#[cfg(not(test))]
#[global_allocator]
static ALLOCATOR: KernelAllocator = KernelAllocator;
