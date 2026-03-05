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

        // Dummy calls for coverage of the stubs
        fd_read(0, 0, 0, 0);
        fd_close(0);
        environ_get(0, 0);
        environ_sizes_get(0, 0);
        args_get(0, 0);
        args_sizes_get(0, 0);
        proc_exit(0);
    }
}

#[link(wasm_import_module = "wasi_snapshot_preview1")]
extern "C" {
    #[link_name = "fd_write"]
    fn fd_write(fd: i32, iovs: i32, iovs_len: i32, nwritten: i32) -> i32;
    #[link_name = "fd_read"]
    fn fd_read(fd: i32, iovs: i32, iovs_len: i32, nread: i32) -> i32;
    #[link_name = "fd_close"]
    fn fd_close(fd: i32) -> i32;
    #[link_name = "environ_get"]
    fn environ_get(environ: i32, environ_buf: i32) -> i32;
    #[link_name = "environ_sizes_get"]
    fn environ_sizes_get(environ_count: i32, environ_buf_size: i32) -> i32;
    #[link_name = "args_get"]
    fn args_get(argv: i32, argv_buf: i32) -> i32;
    #[link_name = "args_sizes_get"]
    fn args_sizes_get(argc: i32, argv_buf_size: i32) -> i32;
    #[link_name = "proc_exit"]
    fn proc_exit(rval: i32);
}
