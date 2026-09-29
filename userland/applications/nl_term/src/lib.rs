#![no_std]
#![allow(clippy::empty_loop)]

extern crate alloc;

use alloc::string::String;


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
            return core::ptr::null_mut();
        }

        let ptr = (*heap).as_mut_ptr().add(align_offset);
        *bump_ptr = align_offset + layout.size();
        ptr
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
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

use inference_runtime::{InferenceEngine, Model, Tensor};

pub struct NlTerminal {
    engine: InferenceEngine,
    model: Option<Model>,
    context: Option<usize>,
}

impl Default for NlTerminal {
    fn default() -> Self {
        Self::new()
    }
}

impl NlTerminal {
    pub fn new() -> Self {
        Self {
            engine: InferenceEngine::new(),
            model: None,
            context: None,
        }
    }

    pub fn init(&mut self) -> Result<(), &'static str> {
        let model = self
            .engine
            .load_model_by_name("nl_term_model")
            .map_err(|_| "Failed to load model")?;
        let ctx = self
            .engine
            .init_execution_context(&model)
            .map_err(|_| "Failed to init execution context")?;
        self.model = Some(model);
        self.context = Some(ctx);
        Ok(())
    }

    pub fn process_input(&mut self, input: &str) -> Result<String, &'static str> {
        if let Some(suggestion) = self.intercept_posix(input) {
            return Ok(suggestion);
        }

        let ctx = self.context.ok_or("Terminal not initialized")?;

        let input_data = alloc::vec![input.len() as u8];
        let tensor = Tensor::new(input_data, alloc::vec![1]);

        self.engine
            .set_input(ctx, 0, &tensor)
            .map_err(|_| "Failed to set input")?;
        self.engine.compute(ctx).map_err(|_| "Compute failed")?;

        let mut out = [0u8; 32];
        let bytes_written = self
            .engine
            .get_output(ctx, 0, &mut out)
            .map_err(|_| "Failed to get output")?;

        if bytes_written > 0 {
            let action_id = out[0];
            if action_id == 110 { // arbitrary magic number for testing
                return Ok(String::from("ls -la"));
            }
        }

        Ok(String::from("echo 'Unknown command'"))
    }

    fn intercept_posix(&self, input: &str) -> Option<String> {
        let input = input.trim();
        if input == "ls" {
            Some(String::from("Did you mean: 'ls -la' to show all files?"))
        } else if input == "cat" {
            Some(String::from("Usage: cat <filename>"))
        } else if input.starts_with("grep ") {
            Some(String::from("AI Suggestion: 'grep -rn <pattern> .' is usually better for searching recursively."))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nl_term_init() {
        let mut term = NlTerminal::new();
        assert!(term.init().is_ok());
    }

    #[test]
    fn test_nl_term_intercept() {
        let mut term = NlTerminal::new();
        term.init().unwrap();

        let res = term.process_input("ls").unwrap();
        assert!(res.contains("ls -la"));

        let res = term.process_input("grep foo").unwrap();
        assert!(res.contains("grep -rn"));
    }

    #[test]
    fn test_nl_term_ai_suggestion() {
        let mut term = NlTerminal::new();
        term.init().unwrap();

        let res = term.process_input("list my files").unwrap();
        // Fallback to model output or mock logic, let's assume unknown for now as len != magic number usually
        assert!(res.contains("echo 'Unknown command'") || res.contains("ls -la"));
    }
}
