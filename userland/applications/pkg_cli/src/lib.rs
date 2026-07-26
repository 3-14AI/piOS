#![no_std]
#![allow(clippy::empty_loop)]

extern crate alloc;
#[cfg(test)]
extern crate std;

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
    let args_vec = vec!["install".to_string(), "hello_world".to_string()]; // fallback

    match run(args_vec) {
        Ok(_) => 0,
        Err(_) => 1,
    }
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
    repo.add_package(Package {
        name: "coreutils".to_string(),
        version: "0.1.0".to_string(),
        dependencies: vec![],
        wasm_blob: vec![],
    });
    repo.add_package(Package {
        name: "nl_sh".to_string(),
        version: "0.1.0".to_string(),
        dependencies: vec!["coreutils".to_string()],
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
        "update" => {
            // Simulate fetching from a remote repository by modifying pm's repo
            pm.repo.add_package(Package {
                name: "new_remote_pkg".to_string(),
                version: "1.0.0".to_string(),
                dependencies: vec![],
                wasm_blob: vec![],
            });
            Ok(())
        }
        "search" => {
            if args.len() < 2 {
                return Err("Missing search term".to_string());
            }
            let search_term = args[1].as_str();
            let mut found = false;
            for (name, _) in pm.repo.packages.iter() {
                if name.contains(search_term) {
                    found = true;
                }
            }
            if found {
                Ok(())
            } else {
                Err("Package not found".to_string())
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

    #[test]
    fn test_cli_update() {
        let args = vec!["update".to_string()];
        let result = run(args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cli_search_missing_term() {
        let args = vec!["search".to_string()];
        let result = run(args);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Missing search term");
    }

    #[test]
    fn test_cli_search_success() {
        let args = vec!["search".to_string(), "core".to_string()];
        let result = run(args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cli_search_failure() {
        let args = vec!["search".to_string(), "nonexistent".to_string()];
        let result = run(args);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Package not found");
    }
}
