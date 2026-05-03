#![no_std]
extern crate alloc;

// We do not export anything globally in test mode.
// Just regular rust functions so the test binary doesn't try to override libc.
// In actual builds, we export `strlen`, etc.

#[cfg(not(test))]
use core::ffi::{c_char, c_void};
#[cfg(not(test))]
use core::ptr;

/// Calculate the length of a string.
/// # Safety
/// The caller must ensure that `s` points to a valid null-terminated string.
#[cfg(not(test))]
#[no_mangle]
pub unsafe extern "C" fn strlen(s: *const c_char) -> usize {
    let mut len = 0;
    if !s.is_null() {
        while *s.add(len) != 0 {
            len += 1;
        }
    }
    len
}

/// Copy a string.
/// # Safety
/// The caller must ensure that `dest` points to a buffer large enough to hold `src`,
/// and that `src` points to a valid null-terminated string.
#[cfg(not(test))]
#[no_mangle]
pub unsafe extern "C" fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char {
    if dest.is_null() || src.is_null() {
        return dest;
    }
    let mut i = 0;
    while *src.add(i) != 0 {
        *dest.add(i) = *src.add(i);
        i += 1;
    }
    *dest.add(i) = 0;
    dest
}

/// Copy memory area.
/// # Safety
/// The caller must ensure that `dest` and `src` point to valid memory buffers of at least `n` bytes.
#[cfg(not(test))]
#[no_mangle]
pub unsafe extern "C" fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void {
    if dest.is_null() || src.is_null() {
        return dest;
    }
    ptr::copy_nonoverlapping(src as *const u8, dest as *mut u8, n);
    dest
}

/// Fill memory with a constant byte.
/// # Safety
/// The caller must ensure that `s` points to a valid memory buffer of at least `n` bytes.
#[cfg(not(test))]
#[no_mangle]
pub unsafe extern "C" fn memset(s: *mut c_void, c: i32, n: usize) -> *mut c_void {
    if s.is_null() {
        return s;
    }
    ptr::write_bytes(s as *mut u8, c as u8, n);
    s
}

#[cfg(not(test))]
use alloc::alloc::{alloc, dealloc, Layout};
#[cfg(not(test))]
const ALIGNMENT: usize = 8;

/// Allocate dynamic memory.
/// # Safety
/// Always returns a pointer to an allocation of size `size` aligned to ALIGNMENT.
#[cfg(not(test))]
#[no_mangle]
pub unsafe extern "C" fn malloc(size: usize) -> *mut c_void {
    if size == 0 {
        return ptr::null_mut();
    }
    let padding = if core::mem::size_of::<usize>() < ALIGNMENT {
        ALIGNMENT
    } else {
        core::mem::size_of::<usize>()
    };

    let layout = Layout::from_size_align(size + padding, ALIGNMENT).unwrap();
    let ptr = alloc(layout);
    if ptr.is_null() {
        return ptr::null_mut();
    }
    *(ptr as *mut usize) = size;
    ptr.add(padding) as *mut c_void
}

/// Free dynamic memory.
/// # Safety
/// The `ptr` must have been allocated by `malloc`.
#[cfg(not(test))]
#[no_mangle]
pub unsafe extern "C" fn free(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    let padding = if core::mem::size_of::<usize>() < ALIGNMENT {
        ALIGNMENT
    } else {
        core::mem::size_of::<usize>()
    };

    let real_ptr = (ptr as *mut u8).sub(padding);
    let size = *(real_ptr as *const usize);
    let layout = Layout::from_size_align(size + padding, ALIGNMENT).unwrap();
    dealloc(real_ptr, layout);
}

#[cfg(test)]
pub mod tests_impl {
    use core::ffi::{c_char, c_void};
    use core::ptr;

    pub unsafe fn strlen(s: *const c_char) -> usize {
        let mut len = 0;
        if !s.is_null() {
            while *s.add(len) != 0 {
                len += 1;
            }
        }
        len
    }

    pub unsafe fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char {
        if dest.is_null() || src.is_null() {
            return dest;
        }
        let mut i = 0;
        while *src.add(i) != 0 {
            *dest.add(i) = *src.add(i);
            i += 1;
        }
        *dest.add(i) = 0;
        dest
    }

    pub unsafe fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void {
        if dest.is_null() || src.is_null() {
            return dest;
        }
        ptr::copy_nonoverlapping(src as *const u8, dest as *mut u8, n);
        dest
    }

    pub unsafe fn memset(s: *mut c_void, c: i32, n: usize) -> *mut c_void {
        if s.is_null() {
            return s;
        }
        ptr::write_bytes(s as *mut u8, c as u8, n);
        s
    }
}

#[cfg(test)]
mod tests {
    use super::tests_impl::*;
    use core::ffi::{c_char, c_void};

    #[test]
    fn test_strlen() {
        unsafe {
            let s = b"hello\0".as_ptr() as *const c_char;
            assert_eq!(strlen(s), 5);
        }
    }

    #[test]
    fn test_strcpy() {
        unsafe {
            let src = b"test\0".as_ptr() as *const c_char;
            let mut dest = [0u8; 10];
            strcpy(dest.as_mut_ptr() as *mut c_char, src);
            assert_eq!(&dest[..5], b"test\0");
        }
    }

    #[test]
    fn test_memcpy() {
        unsafe {
            let src = b"1234567890".as_ptr() as *const c_void;
            let mut dest = [0u8; 10];
            memcpy(dest.as_mut_ptr() as *mut c_void, src, 5);
            assert_eq!(&dest[..5], b"12345");
            assert_eq!(dest[5], 0);
        }
    }

    #[test]
    fn test_memset() {
        unsafe {
            let mut dest = [0u8; 10];
            memset(dest.as_mut_ptr() as *mut c_void, b'A' as i32, 5);
            assert_eq!(&dest[..5], b"AAAAA");
            assert_eq!(dest[5], 0);
        }
    }
}
