#![no_std]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use inference_runtime::InferenceEngine;

pub struct KnowledgeGraph {
    #[allow(dead_code)]
    engine: InferenceEngine,
    nodes: Vec<String>,
    edges: Vec<(usize, usize, String)>,
}

impl KnowledgeGraph {
    pub fn new() -> Self {
        Self {
            engine: InferenceEngine::new(),
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_node(&mut self, data: &str) -> usize {
        self.nodes.push(String::from(data));
        self.nodes.len() - 1
    }

    pub fn add_edge(&mut self, from: usize, to: usize, relation: &str) {
        if from < self.nodes.len() && to < self.nodes.len() {
            self.edges.push((from, to, String::from(relation)));
        }
    }
}

impl Default for KnowledgeGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[cfg(not(test))]
use core::alloc::{GlobalAlloc, Layout};
#[cfg(not(test))]
use core::sync::atomic::{AtomicUsize, Ordering};

#[cfg(not(test))]
struct SimpleAllocator {
    offset: AtomicUsize,
}

#[cfg(not(test))]
const HEAP_SIZE: usize = 1024 * 1024;
#[cfg(not(test))]
#[repr(C, align(4096))]
#[allow(dead_code)]
struct AlignedHeap([u8; HEAP_SIZE]);

#[cfg(not(test))]
struct SyncHeap(core::cell::UnsafeCell<AlignedHeap>);

#[cfg(not(test))]
unsafe impl Sync for SyncHeap {}

#[cfg(not(test))]
static HEAP: SyncHeap = SyncHeap(core::cell::UnsafeCell::new(AlignedHeap([0; HEAP_SIZE])));

#[cfg(not(test))]
unsafe impl GlobalAlloc for SimpleAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let align = layout.align();
        let size = layout.size();
        let mut current_offset = self.offset.load(Ordering::Acquire);
        loop {
            let res = current_offset.next_multiple_of(align);
            let next_offset = res + size;
            if next_offset > HEAP_SIZE {
                return core::ptr::null_mut();
            }
            match self.offset.compare_exchange_weak(
                current_offset,
                next_offset,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    let heap_ptr = HEAP.0.get() as *mut u8;
                    return heap_ptr.add(res);
                }
                Err(new_offset) => {
                    current_offset = new_offset;
                }
            }
        }
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[cfg(not(test))]
#[global_allocator]
static ALLOCATOR: SimpleAllocator = SimpleAllocator {
    offset: AtomicUsize::new(0),
};
