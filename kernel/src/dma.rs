#![allow(unused_imports)]

#[cfg(feature = "verus")]
use vstd::prelude::*;

#[cfg(not(feature = "verus"))]
extern crate alloc;

#[cfg(feature = "verus")]
use verus_state_machines_macros::state_machine;

#[cfg(feature = "verus")]
verus! {

pub struct DmaRegion {
    pub addr: u64,
    pub size: u64,
    pub is_pinned: bool,
}

impl DmaRegion {
    pub closed spec fn pinned(&self) -> bool {
        self.is_pinned
    }

    pub fn new(addr: u64, size: u64) -> (res: Self)
        ensures
            res.addr == addr,
            res.size == size,
            !res.pinned()
    {
        DmaRegion {
            addr,
            size,
            is_pinned: false,
        }
    }

    pub fn pin(&mut self) -> (success: bool)
        ensures
            old(self).pinned() ==> !success && self.pinned() == old(self).pinned(),
            !old(self).pinned() ==> success && self.pinned() == true,
            self.addr == old(self).addr,
            self.size == old(self).size
    {
        if self.is_pinned {
            return false;
        }
        self.is_pinned = true;
        true
    }

    pub fn unpin(&mut self) -> (success: bool)
        ensures
            old(self).pinned() ==> success && !self.pinned(),
            !old(self).pinned() ==> !success && self.pinned() == old(self).pinned(),
            self.addr == old(self).addr,
            self.size == old(self).size
    {
        if !self.is_pinned {
            return false;
        }
        self.is_pinned = false;
        true
    }

    // Abstract IO operation.
    // The requirement is that the region MUST be pinned during I/O.
    pub fn execute_io(&self, data: u8)
        requires
            self.pinned()
    {
        // I/O abstraction ...
    }
}

pub fn test_dma() {
    let mut region = DmaRegion::new(0x1000, 4096);
    assert(!region.pinned());
    assert(region.addr == 0x1000);
    assert(region.size == 4096);

    let pin_res = region.pin();
    assert(pin_res);
    assert(region.pinned());

    // Now it's pinned, we can do I/O
    region.execute_io(42);

    let pin_res2 = region.pin();
    assert(!pin_res2);
    assert(region.pinned());

    let unpin_res = region.unpin();
    assert(unpin_res);
    assert(!region.pinned());

    let unpin_res2 = region.unpin();
    assert(!unpin_res2);
    assert(!region.pinned());
}

} // verus!

#[cfg(not(feature = "verus"))]
#[derive(Debug, PartialEq, Eq)]
pub enum DmaStateStatus {
    Idle,
    Pinned,
}

#[cfg(not(feature = "verus"))]
#[derive(Debug)]
pub struct DmaRegion {
    pub addr: u64,
    pub size: u64,
    pub state: DmaStateStatus,
}

#[cfg(not(feature = "verus"))]
impl DmaRegion {
    pub fn new(addr: u64, size: u64) -> Self {
        Self {
            addr,
            size,
            state: DmaStateStatus::Idle,
        }
    }

    pub fn is_pinned(&self) -> bool {
        self.state == DmaStateStatus::Pinned
    }

    pub fn pin(&mut self) -> bool {
        if self.state == DmaStateStatus::Pinned {
            false
        } else {
            self.state = DmaStateStatus::Pinned;
            true
        }
    }

    pub fn unpin(&mut self) -> bool {
        if self.state == DmaStateStatus::Idle {
            false
        } else {
            self.state = DmaStateStatus::Idle;
            true
        }
    }
}

#[cfg(not(feature = "verus"))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dma_region_new() {
        let region = DmaRegion::new(0x2000, 1024);
        assert_eq!(region.addr, 0x2000);
        assert_eq!(region.size, 1024);
        assert_eq!(region.is_pinned(), false);
    }

    #[test]
    fn test_dma_region_pin_unpin() {
        let mut region = DmaRegion::new(0x3000, 512);

        // Initial state
        assert_eq!(region.is_pinned(), false);

        // Pinning when unpinned should succeed
        assert_eq!(region.pin(), true);
        assert_eq!(region.is_pinned(), true);

        // Pinning again should fail
        assert_eq!(region.pin(), false);
        assert_eq!(region.is_pinned(), true);

        // Unpinning when pinned should succeed
        assert_eq!(region.unpin(), true);
        assert_eq!(region.is_pinned(), false);

        // Unpinning when unpinned should fail
        assert_eq!(region.unpin(), false);
        assert_eq!(region.is_pinned(), false);
    }
}
