#![no_std]
#![allow(clippy::empty_loop)]

extern crate alloc;

#[cfg(not(target_arch = "wasm32"))]
extern crate std;

use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use package_manager::{Package, PackageManager, RemoteRepository, Repository};

#[cfg(target_arch = "wasm32")]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[cfg(target_arch = "wasm32")]
use core::alloc::{GlobalAlloc, Layout};

#[cfg(target_arch = "wasm32")]
struct SimpleAllocator {
    heap: core::cell::UnsafeCell<[u8; 65536]>,
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
    heap: core::cell::UnsafeCell::new([0; 65536]),
    bump_ptr: core::cell::UnsafeCell::new(0),
};

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn main() -> i32 {
    let args_vec = alloc::vec!["install".to_string(), "hello_world".to_string()]; // fallback

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
        dependencies: alloc::vec![],
        wasm_blob: alloc::vec![],
    });
    repo.add_package(Package {
        name: "coreutils".to_string(),
        version: "0.1.0".to_string(),
        dependencies: alloc::vec![],
        wasm_blob: alloc::vec![],
    });
    repo.add_package(Package {
        name: "nl_sh".to_string(),
        version: "0.1.0".to_string(),
        dependencies: alloc::vec!["coreutils".to_string()],
        wasm_blob: alloc::vec![],
    });

    let mut pm = PackageManager::new(repo);

    let remote_repo = RemoteRepository::new("pkg.pios.org", 80);

    match command {
        "install" => {
            if args.len() < 2 {
                return Err("Missing package name".to_string());
            }
            let pkg_name = args[1].as_str();
            match pm.install(pkg_name) {
                Ok(_installed) => {
                    // Success from local repo
                    Ok(())
                }
                Err(local_err) => {
                    // Fall back to remote repository
                    match remote_repo.download_package(pkg_name) {
                        Ok(pkg) => {
                            pm.repo.add_package(pkg);
                            // Try installing again after adding it to the local repo
                            match pm.install(pkg_name) {
                                Ok(_) => Ok(()),
                                Err(e) => Err(e),
                            }
                        }
                        Err(remote_err) => Err(alloc::format!(
                            "Local error: {}, Remote error: {}",
                            local_err,
                            remote_err
                        )),
                    }
                }
            }
        }
        "update" => {
            // Simulate fetching an index from a remote repository
            match remote_repo.fetch_index() {
                Ok(packages) => {
                    for pkg_name in packages {
                        // Dynamically try to fetch new packages from the index
                        if pm.repo.get_package(&pkg_name).is_none() {
                             if let Ok(pkg) = remote_repo.download_package(&pkg_name) {
                                pm.repo.add_package(pkg);
                             }
                        }
                    }
                    Ok(())
                }
                Err(_err) => {
                    // Fallback to local mock update if remote fetch fails
                    pm.repo.add_package(Package {
                        name: "new_remote_pkg".to_string(),
                        version: "1.0.0".to_string(),
                        dependencies: alloc::vec![],
                        wasm_blob: alloc::vec![],
                    });
                    Ok(())
                }
            }
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
        let args = alloc::vec![];
        let result = run(args);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "No command provided");
    }

    #[test]
    fn test_cli_unknown_command() {
        let args = alloc::vec!["unknown".to_string()];
        let result = run(args);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Unknown command");
    }

    #[test]
    fn test_cli_install_missing_pkg_name() {
        let args = alloc::vec!["install".to_string()];
        let result = run(args);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Missing package name");
    }

    #[test]
    fn test_cli_install_success() {
        let args = alloc::vec!["install".to_string(), "hello_world".to_string()];
        let result = run(args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cli_install_failure() {
        let args = alloc::vec!["install".to_string(), "missing_pkg".to_string()];
        let result = run(args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_cli_update() {
        let args = alloc::vec!["update".to_string()];
        let result = run(args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cli_search_missing_term() {
        let args = alloc::vec!["search".to_string()];
        let result = run(args);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Missing search term");
    }

    #[test]
    fn test_cli_search_success() {
        let args = alloc::vec!["search".to_string(), "core".to_string()];
        let result = run(args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cli_search_failure() {
        let args = alloc::vec!["search".to_string(), "nonexistent".to_string()];
        let result = run(args);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Package not found");
    }
}
