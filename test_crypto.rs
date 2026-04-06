#[cfg(feature = "verus")]
use vstd::prelude::*;

#[cfg(feature = "verus")]
verus! {
    pub fn test_xor(x: u8, y: u8) {
        assert(x == x ^ 0);
        assert(x ^ x == 0);
    }
}
