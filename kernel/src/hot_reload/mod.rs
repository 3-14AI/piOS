#[cfg(feature = "verus")]
pub mod verus_impl;

#[cfg(not(feature = "verus"))]
pub mod std_impl;
