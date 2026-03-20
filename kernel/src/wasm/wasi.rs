extern crate alloc;
use alloc::vec::Vec;
use wasmi::Caller;

pub const WASI_ERRNO_SUCCESS: i32 = 0;
pub const WASI_ERRNO_BADF: i32 = 8;
pub const WASI_ERRNO_NOSYS: i32 = 52;

pub struct WasiCtx {
    pub fds: Vec<Option<crate::ipc::RendezvousChannel<Vec<u8>>>>,
    #[cfg(not(feature = "verus"))]
    pub nn_ctx: crate::wasm::wasi_nn::WasiNnCtx,
}

impl Default for WasiCtx {
    fn default() -> Self {
        Self::new()
    }
}

impl WasiCtx {
    pub fn new() -> Self {
        let mut fds = Vec::new();
        // pre-populate fd 0, 1, 2
        for _ in 0..3 {
            fds.push(Some(crate::ipc::RendezvousChannel::new()));
        }
        Self {
            fds,
            #[cfg(not(feature = "verus"))]
            nn_ctx: crate::wasm::wasi_nn::WasiNnCtx::new(),
        }
    }
}

pub fn fd_write(
    mut caller: Caller<'_, WasiCtx>,
    fd: i32,
    iovs_ptr: i32,
    iovs_len: i32,
    nwritten_ptr: i32,
) -> i32 {
    let memory = match caller.get_export("memory").and_then(|e| e.into_memory()) {
        Some(m) => m,
        None => return WASI_ERRNO_BADF,
    };

    let mut total_written = 0;

    for i in 0..iovs_len {
        let offset = (iovs_ptr + i * 8) as usize;
        let mut ptr_buf = [0u8; 4];
        let mut len_buf = [0u8; 4];

        if memory.read(&caller, offset, &mut ptr_buf).is_err() {
            return WASI_ERRNO_BADF;
        }
        if memory.read(&caller, offset + 4, &mut len_buf).is_err() {
            return WASI_ERRNO_BADF;
        }

        let ptr = u32::from_le_bytes(ptr_buf) as usize;
        let len = u32::from_le_bytes(len_buf) as usize;

        let mut data = alloc::vec![0u8; len];
        if memory.read(&caller, ptr, &mut data).is_err() {
            return WASI_ERRNO_BADF;
        }

        if fd == 1 || fd == 2 {
            if let Ok(s) = core::str::from_utf8(&data) {
                log::info!("{}", s);
            }
        }

        let fd_usize = fd as usize;
        let mut sent = false;
        if fd_usize < caller.data().fds.len() {
            if let Some(channel) = &mut caller.data_mut().fds[fd_usize] {
                // Ignore the result of try_send for now as this is a stub/concept
                let _ = channel.try_send(data);
                sent = true;
            }
        }

        if !sent && fd != 1 && fd != 2 {
            return WASI_ERRNO_BADF;
        }

        total_written += len;
    }

    let written_bytes = (total_written as u32).to_le_bytes();
    if memory
        .write(&mut caller, nwritten_ptr as usize, &written_bytes)
        .is_err()
    {
        return WASI_ERRNO_BADF;
    }

    WASI_ERRNO_SUCCESS
}

pub fn fd_read(
    _caller: Caller<'_, WasiCtx>,
    _fd: i32,
    _iovs: i32,
    _iovs_len: i32,
    _nread: i32,
) -> i32 {
    WASI_ERRNO_NOSYS
}

pub fn fd_close(_caller: Caller<'_, WasiCtx>, _fd: i32) -> i32 {
    WASI_ERRNO_NOSYS
}

pub fn environ_get(_caller: Caller<'_, WasiCtx>, _environ: i32, _environ_buf: i32) -> i32 {
    WASI_ERRNO_NOSYS
}

pub fn environ_sizes_get(
    _caller: Caller<'_, WasiCtx>,
    _environ_count: i32,
    _environ_buf_size: i32,
) -> i32 {
    WASI_ERRNO_NOSYS
}

pub fn args_get(_caller: Caller<'_, WasiCtx>, _argv: i32, _argv_buf: i32) -> i32 {
    WASI_ERRNO_NOSYS
}

pub fn args_sizes_get(_caller: Caller<'_, WasiCtx>, _argc: i32, _argv_buf_size: i32) -> i32 {
    WASI_ERRNO_NOSYS
}

pub fn proc_exit(_caller: Caller<'_, WasiCtx>, _rval: i32) {
    // Usually this would trap or terminate the instance
}

#[cfg(not(feature = "verus"))]
pub fn sys_intent(
    mut caller: Caller<'_, WasiCtx>,
    intent_ptr: i32,
    intent_len: i32,
    out_ptr: i32,
    out_max_len: i32,
    nwritten_ptr: i32,
) -> i32 {
    if intent_len < 0 || intent_len > 65536 {
        return WASI_ERRNO_BADF;
    }
    if out_max_len < 0 || out_max_len > 65536 {
        return WASI_ERRNO_BADF;
    }

    let memory = match caller.get_export("memory").and_then(|e| e.into_memory()) {
        Some(m) => m,
        None => return WASI_ERRNO_BADF,
    };

    let mut data = alloc::vec![0u8; intent_len as usize];
    if memory
        .read(&caller, intent_ptr as u32 as usize, &mut data)
        .is_err()
    {
        return WASI_ERRNO_BADF;
    }

    let engine = &mut caller.data_mut().nn_ctx.engine;

    let model = match engine.load_model_by_name("intent_model") {
        Ok(m) => m,
        Err(_) => return WASI_ERRNO_BADF,
    };

    let ctx = match engine.init_execution_context(&model) {
        Ok(c) => c,
        Err(_) => return WASI_ERRNO_BADF,
    };

    let dims = alloc::vec![data.len()];
    let tensor = inference_runtime::Tensor::new(data, dims);

    if engine.set_input(ctx, 0, &tensor).is_err() {
        return WASI_ERRNO_BADF;
    }

    if engine.compute(ctx).is_err() {
        return WASI_ERRNO_BADF;
    }

    let mut out_buf = alloc::vec![0u8; out_max_len as usize];
    let bytes = match engine.get_output(ctx, 0, &mut out_buf) {
        Ok(b) => b,
        Err(_) => return WASI_ERRNO_BADF,
    };

    let out_str = core::str::from_utf8(&out_buf[..bytes]).unwrap_or("");
    log::info!("Intent translated to WASI: {}", out_str);

    if memory
        .write(&mut caller, out_ptr as u32 as usize, &out_buf[..bytes])
        .is_err()
    {
        return WASI_ERRNO_BADF;
    }

    let bytes_u32 = bytes as u32;
    if memory
        .write(
            &mut caller,
            nwritten_ptr as u32 as usize,
            &bytes_u32.to_le_bytes(),
        )
        .is_err()
    {
        return WASI_ERRNO_BADF;
    }

    WASI_ERRNO_SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasi_ctx_init() {
        let ctx = WasiCtx::new();
        assert_eq!(ctx.fds.len(), 3);
        assert!(ctx.fds[0].is_some());
        assert!(ctx.fds[1].is_some());
        assert!(ctx.fds[2].is_some());
        let _ = WasiCtx::default();
    }
}
