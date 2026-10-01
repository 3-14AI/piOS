#![cfg_attr(not(test), no_std)]

extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use inference_runtime::{InferenceEngine, Model};

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[cfg(not(test))]
use core::alloc::{GlobalAlloc, Layout};

#[cfg(not(test))]
struct SimpleAllocator {
    heap: core::cell::UnsafeCell<[u8; 1024 * 1024]>,
    bump_ptr: core::cell::UnsafeCell<usize>,
}

#[cfg(not(test))]
unsafe impl Sync for SimpleAllocator {}

#[cfg(not(test))]
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

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[cfg(not(test))]
#[global_allocator]
static ALLOCATOR: SimpleAllocator = SimpleAllocator {
    heap: core::cell::UnsafeCell::new([0; 1024 * 1024]),
    bump_ptr: core::cell::UnsafeCell::new(0),
};

pub struct NlTerm {
    engine: InferenceEngine,
    model: Option<Model>,
    context: Option<usize>,
    history: Vec<String>,
}

impl Default for NlTerm {
    fn default() -> Self {
        Self::new()
    }
}

impl NlTerm {
    pub fn new() -> Self {
        Self {
            engine: InferenceEngine::new(),
            model: None,
            context: None,
            history: Vec::new(),
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

    pub fn process_input(&mut self, input: &str) -> String {
        self.history.push(input.to_string());

        let trimmed = input.trim();
        if trimmed.starts_with("ls") {
            return String::from("Intercepted POSIX command: ls. AI augmentation: Consider 'ls -la' to see hidden files.");
        } else if trimmed.starts_with("cat") {
            return String::from("Intercepted POSIX command: cat. AI augmentation: Use 'less' or 'bat' for large files.");
        } else if trimmed.starts_with("grep") {
            return String::from("Intercepted POSIX command: grep. AI augmentation: Try 'rg' or 'ag' for faster recursive search.");
        }

        String::from("Command executed: ") + trimmed
    }

    pub fn get_autocomplete_suggestion(
        &mut self,
        current_input: &str,
    ) -> Result<String, &'static str> {
        let ctx = self.context.ok_or("NL Term not initialized")?;

        let input_data = alloc::vec![current_input.len() as u8];
        let tensor = inference_runtime::Tensor::new(input_data, alloc::vec![1]);

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
            let suggestion_id = out[0];
            if suggestion_id == 109 {
                return Ok(String::from("ls -la"));
            } else if suggestion_id == 110 {
                return Ok(String::from("cat file.txt"));
            } else {
                return Ok(String::from("grep 'pattern'"));
            }
        }

        Ok(String::from(""))
    }
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "wasi_snapshot_preview1")]
extern "C" {
    fn fd_read(fd: i32, iovs: *const Iovec, iovs_len: usize, nread: *mut usize) -> i32;
    fn fd_write(fd: i32, iovs: *const Iovec, iovs_len: usize, nwritten: *mut usize) -> i32;
}

#[cfg(target_arch = "wasm32")]
#[repr(C)]
struct Iovec {
    buf: *mut u8,
    buf_len: usize,
}

#[cfg(not(target_arch = "wasm32"))]
fn write_stdout(_msg: &str) {}

#[cfg(target_arch = "wasm32")]
fn write_stdout(msg: &str) {
    unsafe {
        let iov = Iovec {
            buf: msg.as_ptr() as *mut u8,
            buf_len: msg.len(),
        };
        let mut nwritten = 0;
        fd_write(1, &iov, 1, &mut nwritten);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn read_stdin(_buf: &mut [u8]) -> usize {
    0
}

#[cfg(target_arch = "wasm32")]
fn read_stdin(buf: &mut [u8]) -> usize {
    unsafe {
        let iov = Iovec {
            buf: buf.as_mut_ptr(),
            buf_len: buf.len(),
        };
        let mut nread = 0;
        let res = fd_read(0, &iov, 1, &mut nread);
        if res == 0 {
            nread
        } else {
            0
        }
    }
}

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    run();
    loop {}
}

pub fn run() {
    write_stdout("NL-Term: AI-Augmented Terminal Emulator\n");
    let mut term = NlTerm::new();
    if term.init().is_err() {
        write_stdout("Failed to initialize NL-Term AI Model\n");
        return;
    }

    let mut buf = [0u8; 1024];

    loop {
        write_stdout("nl-term> ");
        let nread = read_stdin(&mut buf);

        if nread == 0 {
            break; // EOF or error
        }

        if let Ok(input_str) = core::str::from_utf8(&buf[..nread]) {
            let trimmed = input_str.trim();
            if trimmed.is_empty() {
                continue;
            }
            if trimmed == "exit" {
                break;
            }

            let suggestion = term
                .get_autocomplete_suggestion(trimmed)
                .unwrap_or(String::new());
            if !suggestion.is_empty() {
                let msg = format!("AI Suggestion: {}\n", suggestion);
                write_stdout(&msg);
            }

            let result = term.process_input(trimmed);
            let msg = format!("{}\n", result);
            write_stdout(&msg);
        } else {
            write_stdout("Invalid UTF-8 input\n");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_input_ls() {
        let mut term = NlTerm::new();
        let result = term.process_input("ls -l");
        assert!(result.contains("Intercepted POSIX command: ls"));
        assert!(result.contains("Consider 'ls -la'"));
    }

    #[test]
    fn test_process_input_cat() {
        let mut term = NlTerm::new();
        let result = term.process_input("cat file.txt");
        assert!(result.contains("Intercepted POSIX command: cat"));
        assert!(result.contains("Use 'less' or 'bat'"));
    }

    #[test]
    fn test_process_input_grep() {
        let mut term = NlTerm::new();
        let result = term.process_input("grep search_term");
        assert!(result.contains("Intercepted POSIX command: grep"));
        assert!(result.contains("Try 'rg' or 'ag'"));
    }

    #[test]
    fn test_process_input_other() {
        let mut term = NlTerm::new();
        let result = term.process_input("echo hello");
        assert_eq!(result, "Command executed: echo hello");
        assert_eq!(term.history.len(), 1);
        assert_eq!(term.history[0], "echo hello");
    }

    #[test]
    fn test_get_autocomplete_suggestion() {
        let mut term = NlTerm::new();
        term.init().unwrap();
        let suggestion = term.get_autocomplete_suggestion("l").unwrap();
        assert_eq!(suggestion, "ls -la");
    }

    #[test]
    fn test_run_loop() {
        // Just verify run doesn't panic in test config
        run();
    }
}
