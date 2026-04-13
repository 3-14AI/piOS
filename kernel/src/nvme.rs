#![allow(dead_code)]

#[cfg(feature = "verus")]
use vstd::prelude::*;

#[cfg(feature = "verus")]
verus! {
    pub struct NvmeDriver {
        pub capacity: u64,
    }

    impl NvmeDriver {
        pub fn new(capacity: u64) -> (d: Self)
            ensures d.capacity == capacity
        {
            NvmeDriver { capacity }
        }
    }
}

#[cfg(not(feature = "verus"))]
pub struct NvmeDriver {
    pub capacity: u64,
}

#[cfg(not(feature = "verus"))]
impl NvmeDriver {
    pub fn new(capacity: u64) -> Self {
        NvmeDriver { capacity }
    }
}

#[cfg(not(feature = "verus"))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nvme_driver() {
        let drv = NvmeDriver::new(1024);
        assert_eq!(drv.capacity, 1024);
    }
}
