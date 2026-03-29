#[cfg(feature = "verus")]
pub mod verus_impl;

#[cfg(feature = "verus")]
pub use verus_impl::*;

#[cfg(not(feature = "verus"))]
pub mod std_impl;

#[cfg(not(feature = "verus"))]
pub use std_impl::*;
