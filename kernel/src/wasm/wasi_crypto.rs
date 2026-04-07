extern crate alloc;

#[cfg(feature = "verus")]
use vstd::prelude::*;

pub struct WasiCryptoCtx {
    // Basic state for the crypto context
    pub is_initialized: bool,
}

impl Default for WasiCryptoCtx {
    fn default() -> Self {
        Self::new()
    }
}

impl WasiCryptoCtx {
    pub fn new() -> Self {
        Self {
            is_initialized: true,
        }
    }
}

#[cfg(feature = "verus")]
verus! {
    proof fn bitwise_xor_eq(x: u8, y: u8)
        ensures (x ^ y == 0) <==> (x == y)
    {
        assert((x ^ y == 0) <==> (x == y)) by(bit_vector);
    }

    proof fn bitwise_or_eq(x: u8, y: u8)
        ensures (x | y == 0) <==> (x == 0 && y == 0)
    {
        assert((x | y == 0) <==> (x == 0 && y == 0)) by(bit_vector);
    }

    pub fn constant_time_eq(a: &[u8], b: &[u8]) -> (res: u8)
        requires
            a@.len() == b@.len(),
            a@.len() < 0x100000000,
        ensures
            res == 1 <==> a@ =~= b@,
            res == 0 <==> !(a@ =~= b@),
            res == 1 || res == 0,
    {
        let mut diff: u8 = 0;
        let mut i: usize = 0;
        let len = a.len();

        while i < len
            invariant
                len == a@.len(),
                len == b@.len(),
                i <= len,
                diff == 0 <==> (forall|k: int| 0 <= k && k < i ==> a@[k] == b@[k]),
            decreases len - i,
        {
            let a_val = a[i];
            let b_val = b[i];
            let bit_diff = a_val ^ b_val;

            proof {
                bitwise_xor_eq(a_val, b_val);
                bitwise_or_eq(diff, bit_diff);
            }

            diff = diff | bit_diff;

            i += 1;
        }

        if diff == 0 {
            assert(a@ =~= b@);
            1
        } else {
            assert(!(a@ =~= b@));
            0
        }
    }
}

#[cfg(not(feature = "verus"))]
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> u8 {
    if a.len() != b.len() {
        return 0;
    }

    let mut diff: u8 = 0;
    for i in 0..a.len() {
        diff |= a[i] ^ b[i];
    }

    if diff == 0 {
        1
    } else {
        0
    }
}

#[cfg(not(feature = "verus"))]
pub fn constant_time_eq_host(
    mut caller: wasmi::Caller<'_, crate::wasm::wasi::WasiCtx>,
    a_ptr: i32,
    b_ptr: i32,
    len: i32,
    res_ptr: i32,
) -> i32 {
    let memory = match caller.get_export("memory").and_then(|e| e.into_memory()) {
        Some(m) => m,
        None => return crate::wasm::wasi::WASI_ERRNO_BADF,
    };

    let u_len = len as u32 as usize;
    // Bound check to prevent DoS attacks
    if u_len > 1024 * 1024 {
        return crate::wasm::wasi::WASI_ERRNO_BADF;
    }

    let mut a_buf = alloc::vec![0u8; u_len];
    if memory
        .read(&caller, a_ptr as u32 as usize, &mut a_buf)
        .is_err()
    {
        return crate::wasm::wasi::WASI_ERRNO_BADF;
    }

    let mut b_buf = alloc::vec![0u8; u_len];
    if memory
        .read(&caller, b_ptr as u32 as usize, &mut b_buf)
        .is_err()
    {
        return crate::wasm::wasi::WASI_ERRNO_BADF;
    }

    let res = constant_time_eq(&a_buf, &b_buf);

    if memory
        .write(&mut caller, res_ptr as u32 as usize, &[res])
        .is_err()
    {
        return crate::wasm::wasi::WASI_ERRNO_BADF;
    }

    crate::wasm::wasi::WASI_ERRNO_SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasi_crypto_ctx_init() {
        let ctx = WasiCryptoCtx::new();
        assert!(ctx.is_initialized);
        let _ = WasiCryptoCtx::default();
    }

    #[test]
    fn test_constant_time_eq() {
        let a = [1, 2, 3];
        let b = [1, 2, 3];
        let c = [1, 2, 4];
        let d = [1, 2];

        assert_eq!(constant_time_eq(&a, &b), 1);
        assert_eq!(constant_time_eq(&a, &c), 0);
        assert_eq!(constant_time_eq(&a, &d), 0);
    }
}
