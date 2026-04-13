#![allow(dead_code)]

#[cfg(feature = "verus")]
use vstd::prelude::*;

#[cfg(feature = "verus")]
verus! {
    pub struct AmdIntelGpuDriver {
        pub vram_size: u64,
    }

    impl AmdIntelGpuDriver {
        pub fn new(vram_size: u64) -> (d: Self)
            ensures d.vram_size == vram_size
        {
            AmdIntelGpuDriver { vram_size }
        }
    }
}

#[cfg(not(feature = "verus"))]
pub struct AmdIntelGpuDriver {
    pub vram_size: u64,
}

#[cfg(not(feature = "verus"))]
impl AmdIntelGpuDriver {
    pub fn new(vram_size: u64) -> Self {
        AmdIntelGpuDriver { vram_size }
    }
}

#[cfg(not(feature = "verus"))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_driver() {
        let drv = AmdIntelGpuDriver::new(4096);
        assert_eq!(drv.vram_size, 4096);
    }
}
