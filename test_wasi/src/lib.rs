#![no_std]
#![no_main]

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn main() {
    unsafe {
        let text = b"hello\n";
        let iov = [text.as_ptr() as u32, text.len() as u32];
        let ptr = iov.as_ptr() as i32;
        let mut nwritten = 0i32;
        fd_write(1, ptr, 1, &mut nwritten as *mut _ as i32);
    }
}

#[link(wasm_import_module = "wasi_snapshot_preview1")]
extern "C" {
    #[link_name = "fd_write"]
    fn fd_write(fd: i32, iovs: i32, iovs_len: i32, nwritten: i32) -> i32;
}
