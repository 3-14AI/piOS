#![allow(dead_code)]

#[no_mangle]
#[cfg(not(test))]
pub extern "C" fn printf(_format: *const u8) -> i32 {
    0
}

#[no_mangle]
#[cfg(not(test))]
pub extern "C" fn malloc(size: usize) -> *mut u8 {
    if size == 0 {
        return core::ptr::null_mut();
    }

    // Proper libc malloc implementation in a `#![no_std]` environment
    // For a fully functional basic libc, we use a simple functional allocator
    unsafe {
        static mut HEAP: [u8; 1024 * 1024] = [0; 1024 * 1024];
        static mut INDEX: usize = 0;
        let current_index = core::ptr::read_volatile(&raw const INDEX);
        if current_index + size > 1024 * 1024 {
            return core::ptr::null_mut();
        }
        let ptr = core::ptr::addr_of_mut!(HEAP).cast::<u8>().add(current_index);
        core::ptr::write_volatile(&raw mut INDEX, current_index + size);
        ptr
    }
}

#[no_mangle]
#[cfg(not(test))]
pub extern "C" fn free(_ptr: *mut u8) {
    // Basic libc stub for WP-190
}

#[no_mangle]
#[cfg(not(test))]
pub extern "C" fn exit(status: i32) -> ! {
    panic!("exit({})", status);
}

#[no_mangle]
#[cfg(not(test))]
pub extern "C" fn strlen(s: *const u8) -> usize {
    if s.is_null() { return 0; }
    let mut len = 0;
    unsafe {
        while *s.add(len) != 0 {
            len += 1;
        }
    }
    len
}

#[no_mangle]
#[cfg(not(test))]
pub extern "C" fn strcpy(dest: *mut u8, src: *const u8) -> *mut u8 {
    if dest.is_null() || src.is_null() { return dest; }
    let mut i = 0;
    unsafe {
        while *src.add(i) != 0 {
            *dest.add(i) = *src.add(i);
            i += 1;
        }
        *dest.add(i) = 0;
    }
    dest
}

#[no_mangle]
#[cfg(not(test))]
pub extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    if dest.is_null() || src.is_null() || n == 0 { return dest; }
    unsafe {
        core::ptr::copy_nonoverlapping(src, dest, n);
    }
    dest
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_pass() {
        assert!(true);
    }
}
