#![no_std]
#![allow(clippy::empty_loop)]

extern crate alloc;


use core::alloc::{GlobalAlloc, Layout};

#[cfg(target_arch = "wasm32")]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

struct SimpleAllocator {
    heap: core::cell::UnsafeCell<[u8; 65536]>,
    bump_ptr: core::cell::UnsafeCell<usize>,
}

unsafe impl Sync for SimpleAllocator {}

unsafe impl GlobalAlloc for SimpleAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let bump_ptr = self.bump_ptr.get();
        let heap = self.heap.get();

        let align_offset = (*bump_ptr).wrapping_add(layout.align() - 1) & !(layout.align() - 1);

        if align_offset + layout.size() > (*heap).len() {
            return core::ptr::null_mut(); // Out of memory
        }

        let ptr = (*heap).as_mut_ptr().add(align_offset);
        *bump_ptr = align_offset + layout.size();
        ptr
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {} // Memory leak by design
}

#[global_allocator]
static ALLOCATOR: SimpleAllocator = SimpleAllocator {
    heap: core::cell::UnsafeCell::new([0; 65536]),
    bump_ptr: core::cell::UnsafeCell::new(0),
};


#[link(wasm_import_module = "wasi_ephemeral_nn")]
extern "C" {
    pub fn load(
        builder_ptr: *const u8,
        builder_len: i32,
        encoding: i32,
        target: i32,
        graph_ptr: *mut u32,
    ) -> i32;
    pub fn load_by_name(name_ptr: *const u8, name_len: i32, graph_ptr: *mut u32) -> i32;
    pub fn init_execution_context(graph: u32, context_ptr: *mut u32) -> i32;
    pub fn set_input(context: u32, index: i32, tensor_ptr: *const u8) -> i32;
    pub fn compute(context: u32) -> i32;
    pub fn get_output(
        context: u32,
        index: i32,
        out_buffer_ptr: *mut u8,
        out_buffer_max_size: i32,
        bytes_written_ptr: *mut u32,
    ) -> i32;
}

#[no_mangle]
pub extern "C" fn main() -> i32 {
    let mut graph: u32 = 0;
    let name = b"mistral_model";

    // Load model
    let res = unsafe {
        load_by_name(name.as_ptr(), name.len() as i32, &mut graph as *mut u32)
    };
    if res != 0 {
        return res;
    }

    // Init execution context
    let mut context: u32 = 0;
    let res = unsafe {
        init_execution_context(graph, &mut context as *mut u32)
    };
    if res != 0 {
        return res;
    }

    // Set input
    let dummy_input = [0u8; 10];
    let res = unsafe {
        set_input(context, 0, dummy_input.as_ptr())
    };
    if res != 0 {
        return res;
    }

    // Compute
    let res = unsafe {
        compute(context)
    };
    if res != 0 {
        return res;
    }

    // Get output
    let mut out_buffer = [0u8; 128];
    let mut bytes_written: u32 = 0;
    let res = unsafe {
        get_output(
            context,
            0,
            out_buffer.as_mut_ptr(),
            out_buffer.len() as i32,
            &mut bytes_written as *mut u32,
        )
    };

    if res != 0 {
        return res;
    }

    0 // Success
}
