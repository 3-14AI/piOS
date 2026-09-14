#![no_std]
#![allow(clippy::empty_loop)]

extern crate alloc;

#[no_mangle]
pub unsafe extern "C" fn strlen(s: *const u8) -> usize {
    let mut len = 0;
    if s.is_null() {
        return 0;
    }
    while *s.add(len) != 0 {
        len += 1;
    }
    len
}

#[no_mangle]
pub unsafe extern "C" fn strcpy(dest: *mut u8, src: *const u8) -> *mut u8 {
    if dest.is_null() || src.is_null() {
        return dest;
    }
    let mut i = 0;
    loop {
        let c = *src.add(i);
        *dest.add(i) = c;
        if c == 0 {
            break;
        }
        i += 1;
    }
    dest
}

#[no_mangle]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    if dest.is_null() || src.is_null() {
        return dest;
    }
    core::ptr::copy_nonoverlapping(src, dest, n);
    dest
}

#[no_mangle]
pub unsafe extern "C" fn memset(dest: *mut u8, c: i32, n: usize) -> *mut u8 {
    if dest.is_null() {
        return dest;
    }
    core::ptr::write_bytes(dest, c as u8, n);
    dest
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strlen() {
        unsafe {
            let s = b"hello\0";
            assert_eq!(strlen(s.as_ptr()), 5);
        }
    }

    #[test]
    fn test_strcpy() {
        unsafe {
            let src = b"test\0";
            let mut dest = [0u8; 10];
            strcpy(dest.as_mut_ptr(), src.as_ptr());
            assert_eq!(&dest[0..5], b"test\0");
        }
    }

    #[test]
    fn test_memcpy() {
        unsafe {
            let src = b"12345";
            let mut dest = [0u8; 5];
            memcpy(dest.as_mut_ptr(), src.as_ptr(), 5);
            assert_eq!(&dest, b"12345");
        }
    }

    #[test]
    fn test_memset() {
        unsafe {
            let mut dest = [0u8; 5];
            memset(dest.as_mut_ptr(), 0xAA, 5);
            assert_eq!(&dest, &[0xAA, 0xAA, 0xAA, 0xAA, 0xAA]);
        }
    }
}
