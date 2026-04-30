#![no_std]
#![allow(clippy::empty_loop)]

extern crate alloc;

use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use package_manager::{Package, PackageManager, Repository};

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[cfg(not(test))]
use core::alloc::{GlobalAlloc, Layout};

#[cfg(not(test))]
struct DummyAllocator;

#[cfg(not(test))]
unsafe impl GlobalAlloc for DummyAllocator {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        core::ptr::null_mut()
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[cfg(not(test))]
#[global_allocator]
static ALLOCATOR: DummyAllocator = DummyAllocator;

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    let args = vec!["install".to_string(), "hello_world".to_string()];
    let _ = run(args);
    loop {}
}

pub fn main() {
    let args = vec!["install".to_string(), "hello_world".to_string()];
    let _ = run(args);
}

pub fn run(args: Vec<String>) -> Result<(), String> {
    if args.is_empty() {
        return Err("No command provided".to_string());
    }

    let command = args[0].as_str();

    let mut repo = Repository::new();
    repo.add_package(Package {
        name: "hello_world".to_string(),
        version: "1.0.0".to_string(),
        dependencies: vec![],
        wasm_blob: vec![],
    });

    let mut pm = PackageManager::new(repo);

    match command {
        "install" => {
            if args.len() < 2 {
                return Err("Missing package name".to_string());
            }
            let pkg_name = args[1].as_str();
            match pm.install(pkg_name) {
                Ok(_installed) => {
                    // Simulating success
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }
        _ => Err("Unknown command".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_no_args() {
        let args = vec![];
        let result = run(args);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "No command provided");
    }

    #[test]
    fn test_cli_unknown_command() {
        let args = vec!["unknown".to_string()];
        let result = run(args);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Unknown command");
    }

    #[test]
    fn test_cli_install_missing_pkg_name() {
        let args = vec!["install".to_string()];
        let result = run(args);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Missing package name");
    }

    #[test]
    fn test_cli_install_success() {
        let args = vec!["install".to_string(), "hello_world".to_string()];
        let result = run(args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cli_install_failure() {
        let args = vec!["install".to_string(), "missing_pkg".to_string()];
        let result = run(args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }
}
