#![no_main]

use cranelift_codegen::{
    ir::{types, AbiParam, Function, InstBuilder, Signature},
    isa::CallConv,
};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
// use cranelift_native::builder as host_isa_builder; // not using native in wasi

#[no_mangle]
pub extern "C" fn main() {
    unsafe {
        let text = b"hello from cranelift integration\n";
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

        let mut sig = Signature::new(CallConv::SystemV);
        sig.returns.push(AbiParam::new(types::I32));
        sig.params.push(AbiParam::new(types::I32));

        let mut fn_builder_ctx = FunctionBuilderContext::new();
        let mut func =
            Function::with_name_signature(cranelift_codegen::ir::UserFuncName::user(0, 0), sig);
        let mut builder = FunctionBuilder::new(&mut func, &mut fn_builder_ctx);

        let block0 = builder.create_block();
        builder.append_block_params_for_function_params(block0);
        builder.switch_to_block(block0);
        builder.seal_block(block0);

        let x = builder.block_params(block0)[0];
        let y = builder.ins().iconst(types::I32, 42);
        let sum = builder.ins().iadd(x, y);
        builder.ins().return_(&[sum]);

        builder.finalize();

        let text2 = b"cranelift build complete\n";
        let iov2 = [text2.as_ptr() as u32, text2.len() as u32];
        let ptr2 = iov2.as_ptr() as i32;
        fd_write(1, ptr2, 1, &mut nwritten as *mut _ as i32);

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
