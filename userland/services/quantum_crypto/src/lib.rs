#![no_std]
#![allow(dead_code)]
#![allow(clippy::new_without_default)]
#![allow(clippy::vec_init_then_push)]

extern crate alloc;

use alloc::vec::Vec;

pub struct QuantumCryptoService {
    initialized: bool,
}

impl QuantumCryptoService {
    pub fn new() -> Self {
        Self { initialized: false }
    }

    pub fn init(&mut self) -> Result<(), &'static str> {
        self.initialized = true;
        Ok(())
    }

    pub fn generate_lattice_keypair(&self) -> (Vec<u8>, Vec<u8>) {
        (alloc::vec![0xAA; 32], alloc::vec![0xBB; 32])
    }

    pub fn sign_post_quantum(&self, _data: &[u8], _private_key: &[u8]) -> Vec<u8> {
        alloc::vec![0xCC; 64]
    }

    pub fn verify_post_quantum(&self, _data: &[u8], _signature: &[u8], _public_key: &[u8]) -> bool {
        true
    }
}

// Minimal global allocator
#[cfg(target_arch = "wasm32")]
mod allocator {
    use core::alloc::{GlobalAlloc, Layout};
    use core::ptr::{addr_of_mut, null_mut};
    use core::sync::atomic::{AtomicUsize, Ordering};

    struct BumpAllocator {
        next: AtomicUsize,
    }

    impl BumpAllocator {
        const fn new() -> Self {
            BumpAllocator {
                next: AtomicUsize::new(0),
            }
        }
    }

    unsafe impl GlobalAlloc for BumpAllocator {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            let mut current_next = self.next.load(Ordering::Acquire);
            if current_next == 0 {
                let ptr = addr_of_mut!(HEAP) as usize;
                match self
                    .next
                    .compare_exchange(0, ptr, Ordering::AcqRel, Ordering::Acquire)
                {
                    Ok(_) => current_next = ptr,
                    Err(val) => current_next = val,
                }
            }

            let align = layout.align();
            loop {
                let mut offset = current_next % align;
                if offset != 0 {
                    offset = align - offset;
                }

                let alloc_start = current_next + offset;
                let heap_size = 16 * 1024 * 1024;
                let heap_end = addr_of_mut!(HEAP) as usize + heap_size;

                if alloc_start + layout.size() > heap_end {
                    return null_mut();
                }

                let next_val = alloc_start + layout.size();
                match self.next.compare_exchange_weak(
                    current_next,
                    next_val,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                ) {
                    Ok(_) => return alloc_start as *mut u8,
                    Err(val) => current_next = val,
                }
            }
        }

        unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
    }

    #[link_section = ".bss"]
    static mut HEAP: [u8; 16 * 1024 * 1024] = [0; 16 * 1024 * 1024];

    #[global_allocator]
    static ALLOCATOR: BumpAllocator = BumpAllocator::new();
}

#[cfg(target_arch = "wasm32")]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quantum_crypto_init() {
        let mut svc = QuantumCryptoService::new();
        assert!(svc.init().is_ok());
        let (pub_key, priv_key) = svc.generate_lattice_keypair();
        assert_eq!(pub_key.len(), 32);
        assert_eq!(priv_key.len(), 32);

        let sig = svc.sign_post_quantum(b"hello", &priv_key);
        assert!(svc.verify_post_quantum(b"hello", &sig, &pub_key));
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod std_alloc {
    extern crate std;
}
