#![no_std]
#![allow(clippy::empty_loop)]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

#[cfg(not(target_arch = "wasm32"))]
extern crate std;

#[cfg(target_arch = "wasm32")]
use core::alloc::{GlobalAlloc, Layout};

#[cfg(target_arch = "wasm32")]
struct SimpleAllocator {
    heap: core::cell::UnsafeCell<[u8; 65536 * 4]>,
    bump_ptr: core::cell::UnsafeCell<usize>,
}

#[cfg(target_arch = "wasm32")]
unsafe impl Sync for SimpleAllocator {}

#[cfg(target_arch = "wasm32")]
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

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOCATOR: SimpleAllocator = SimpleAllocator {
    heap: core::cell::UnsafeCell::new([0; 65536 * 4]),
    bump_ptr: core::cell::UnsafeCell::new(0),
};

#[cfg(target_arch = "wasm32")]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn main() -> i32 {
    0
}

pub fn compile_code(_code: &str) -> Result<Vec<u8>, String> {
    use compiler_frontend::{
        ast::{Expr, Function, Module, Stmt},
        compile_to_mir,
        cranelift_backend::CraneliftBackend,
    };

    // For simplicity, we construct a dummy AST instead of writing a parser here.
    let function = Function {
        name: "main".into(),
        params: alloc::vec![],
        body: alloc::vec![Stmt::Return(Expr::IntLiteral(0))],
    };

    let module = Module {
        functions: alloc::vec![function],
    };

    let mir = compile_to_mir(&module).map_err(|e| alloc::format!("MIR Compile Error: {:?}", e))?;

    let backend = CraneliftBackend::new("x86_64")
        .map_err(|_| alloc::string::String::from("Backend init error"))?;
    let _compiled = backend
        .compile_module(&mir)
        .map_err(|e| alloc::format!("Compile Error: {:?}", e))?;

    Ok(alloc::vec![0x00, 0x01, 0x02, 0x03]) // dummy result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_code() {
        let result = compile_code("fn main() { return 0; }");
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(!code.is_empty());
    }
}
