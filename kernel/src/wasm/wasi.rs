extern crate alloc;
use alloc::vec::Vec;
use wasmi::Caller;

pub const WASI_ERRNO_SUCCESS: i32 = 0;
pub const WASI_ERRNO_BADF: i32 = 8;
pub const WASI_ERRNO_NOSYS: i32 = 52;

pub struct WasiCtx {
    pub fds: Vec<Option<crate::ipc::RendezvousChannel<Vec<u8>>>>,
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
        Self { fds }
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
    }
}
