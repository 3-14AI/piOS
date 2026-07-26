#![no_std]
#![allow(clippy::empty_loop)]

extern crate alloc;
#[cfg(test)]
extern crate std;

use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[cfg(not(test))]
use core::alloc::{GlobalAlloc, Layout};

#[cfg(not(test))]
struct SimpleAllocator {
    heap: core::cell::UnsafeCell<[u8; 65536]>,
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
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {} // Memory leak by design
}

#[cfg(not(test))]
#[global_allocator]
static ALLOCATOR: SimpleAllocator = SimpleAllocator {
    heap: core::cell::UnsafeCell::new([0; 65536]),
    bump_ptr: core::cell::UnsafeCell::new(0),
};

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn main() -> i32 {
    let args_vec = vec!["ls".to_string()];

    match run(args_vec) {
        Ok(_) => 0,
        Err(_) => 1,
    }
}

pub fn run(args: Vec<String>) -> Result<String, String> {
    if args.is_empty() {
        return Err("No command provided".to_string());
    }

    let command = args[0].as_str();

    match command {
        "ls" => ls(&args[1..]),
        "cat" => cat(&args[1..]),
        "mkdir" => mkdir(&args[1..]),
        "rm" => rm(&args[1..]),
        "ps" => ps(&args[1..]),
        "kill" => kill(&args[1..]),
        _ => Err(format!("Unknown command: {}", command)),
    }
}

fn ls(_args: &[String]) -> Result<String, String> {
    Ok(".\n..".to_string())
}

fn cat(args: &[String]) -> Result<String, String> {
    if args.is_empty() {
        return Err("cat: missing operand".to_string());
    }
    Ok(format!("Content of {}", args[0]))
}

fn mkdir(args: &[String]) -> Result<String, String> {
    if args.is_empty() {
        return Err("mkdir: missing operand".to_string());
    }
    Ok(format!("Created directory {}", args[0]))
}

fn rm(args: &[String]) -> Result<String, String> {
    if args.is_empty() {
        return Err("rm: missing operand".to_string());
    }
    Ok(format!("Removed {}", args[0]))
}

fn ps(_args: &[String]) -> Result<String, String> {
    Ok("PID TTY TIME CMD\n1 ? 00:00:00 init".to_string())
}

fn kill(args: &[String]) -> Result<String, String> {
    if args.is_empty() {
        return Err("kill: usage: kill pid".to_string());
    }
    Ok(format!("Killed process {}", args[0]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ls() {
        let args = vec!["ls".to_string()];
        assert_eq!(run(args).unwrap(), ".\n..");
    }

    #[test]
    fn test_cat() {
        let args = vec!["cat".to_string(), "file.txt".to_string()];
        assert_eq!(run(args).unwrap(), "Content of file.txt");
    }

    #[test]
    fn test_cat_missing_operand() {
        let args = vec!["cat".to_string()];
        assert!(run(args).is_err());
    }

    #[test]
    fn test_mkdir() {
        let args = vec!["mkdir".to_string(), "new_dir".to_string()];
        assert_eq!(run(args).unwrap(), "Created directory new_dir");
    }

    #[test]
    fn test_rm() {
        let args = vec!["rm".to_string(), "file.txt".to_string()];
        assert_eq!(run(args).unwrap(), "Removed file.txt");
    }

    #[test]
    fn test_ps() {
        let args = vec!["ps".to_string()];
        assert_eq!(run(args).unwrap(), "PID TTY TIME CMD\n1 ? 00:00:00 init");
    }

    #[test]
    fn test_kill() {
        let args = vec!["kill".to_string(), "1".to_string()];
        assert_eq!(run(args).unwrap(), "Killed process 1");
    }

    #[test]
    fn test_unknown_command() {
        let args = vec!["unknown".to_string()];
        assert!(run(args).is_err());
    }
}
