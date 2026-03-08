#![allow(unused_imports)]
extern crate alloc;
use alloc::vec::Vec;

#[cfg(feature = "verus")]
use vstd::prelude::*;

#[cfg(feature = "verus")]
verus! {
    /// VirtIO Descriptor
    #[derive(Copy, Clone)]
    pub struct Descriptor {
        pub addr: u64,
        pub len: u32,
        pub flags: u16,
        pub next: u16,
    }

    /// Available Ring
    #[derive(Clone)]
    pub struct AvailRing {
        pub flags: u16,
        pub idx: u16,
        pub ring: Vec<u16>,
    }

    /// Used Element
    #[derive(Copy, Clone)]
    pub struct UsedElem {
        pub id: u32,
        pub len: u32,
    }

    /// Used Ring
    #[derive(Clone)]
    pub struct UsedRing {
        pub flags: u16,
        pub idx: u16,
        pub ring: Vec<UsedElem>,
    }

    /// Virtqueue abstraction
    pub struct Virtqueue {
        pub queue_size: u16,
        pub descriptors: Vec<Descriptor>,
        pub avail: AvailRing,
        pub used: UsedRing,
        pub last_used_idx: u16,
    }

    impl Virtqueue {
        pub fn new(size: u16) -> (v: Self)
            requires
                size > 0,
            ensures
                v.queue_size == size,
                v.descriptors.len() == size as int,
                v.avail.ring.len() == size as int,
                v.used.ring.len() == size as int,
                v.last_used_idx == 0,
                v.avail.idx == 0,
                v.used.idx == 0
        {
            let mut descriptors = Vec::new();
            let mut avail_ring = Vec::new();
            let mut used_ring = Vec::new();

            let mut i = 0;
            while i < size
                invariant
                    0 <= i && i <= size,
                    descriptors.len() == i as int,
                    avail_ring.len() == i as int,
                    used_ring.len() == i as int
                decreases size - i
            {
                descriptors.push(Descriptor { addr: 0, len: 0, flags: 0, next: 0 });
                avail_ring.push(0);
                used_ring.push(UsedElem { id: 0, len: 0 });
                i = i + 1;
            }

            Virtqueue {
                queue_size: size,
                descriptors,
                avail: AvailRing { flags: 0, idx: 0, ring: avail_ring },
                used: UsedRing { flags: 0, idx: 0, ring: used_ring },
                last_used_idx: 0,
            }
        }

        /// Adds a buffer to the available ring.
        /// Returns true if successful, false if the queue is full.
        pub fn add_avail(&mut self, desc_idx: u16) -> (success: bool)
            requires
                old(self).queue_size > 0,
                old(self).avail.ring.len() == old(self).queue_size as int,
                desc_idx < old(self).queue_size
            ensures
                self.queue_size == old(self).queue_size,
                self.avail.ring.len() == old(self).avail.ring.len(),
                self.avail.ring.len() == self.queue_size as int,
                success ==> self.avail.idx == (old(self).avail.idx + 1),
                !success ==> self.avail.idx == old(self).avail.idx
        {
            // Simple check to avoid wrapping past u16::MAX
            if self.avail.idx == 0xffff {
                return false;
            }
            let avail_idx = self.avail.idx;
            let ring_idx = (avail_idx as u32 % self.queue_size as u32) as usize;

            self.avail.ring.set(ring_idx, desc_idx);
            self.avail.idx = self.avail.idx + 1;
            true
        }

        /// Gets a used buffer.
        /// Returns Some(UsedElem) if there are new used buffers, None otherwise.
        pub fn get_used(&mut self) -> (res: Option<UsedElem>)
            requires
                old(self).queue_size > 0,
                old(self).used.ring.len() == old(self).queue_size as int
            ensures
                self.queue_size == old(self).queue_size,
                self.used.ring.len() == old(self).used.ring.len(),
                self.used.ring.len() == self.queue_size as int,
                match res {
                    Some(_) => self.last_used_idx == (old(self).last_used_idx + 1) && old(self).last_used_idx < 0xffff,
                    None => self.last_used_idx == old(self).last_used_idx
                }
        {
            if self.last_used_idx == self.used.idx {
                return None;
            }
            if self.last_used_idx == 0xffff {
                return None;
            }

            let last_idx = self.last_used_idx;
            let ring_idx = (last_idx as u32 % self.queue_size as u32) as usize;

            let elem = self.used.ring[ring_idx];
            self.last_used_idx = self.last_used_idx + 1;
            Some(elem)
        }
    }
}

#[cfg(not(feature = "verus"))]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Descriptor {
    pub addr: u64,
    pub len: u32,
    pub flags: u16,
    pub next: u16,
}

#[cfg(not(feature = "verus"))]
#[derive(Clone, Debug)]
pub struct AvailRing {
    pub flags: u16,
    pub idx: u16,
    pub ring: alloc::vec::Vec<u16>,
}

#[cfg(not(feature = "verus"))]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct UsedElem {
    pub id: u32,
    pub len: u32,
}

#[cfg(not(feature = "verus"))]
#[derive(Clone, Debug)]
pub struct UsedRing {
    pub flags: u16,
    pub idx: u16,
    pub ring: alloc::vec::Vec<UsedElem>,
}

#[cfg(not(feature = "verus"))]
#[derive(Debug)]
pub struct Virtqueue {
    pub queue_size: u16,
    pub descriptors: alloc::vec::Vec<Descriptor>,
    pub avail: AvailRing,
    pub used: UsedRing,
    pub last_used_idx: u16,
}

#[cfg(not(feature = "verus"))]
impl Virtqueue {
    pub fn new(size: u16) -> Self {
        assert!(size > 0);
        Virtqueue {
            queue_size: size,
            descriptors: alloc::vec![Descriptor { addr: 0, len: 0, flags: 0, next: 0 }; size as usize],
            avail: AvailRing { flags: 0, idx: 0, ring: alloc::vec![0; size as usize] },
            used: UsedRing { flags: 0, idx: 0, ring: alloc::vec![UsedElem { id: 0, len: 0 }; size as usize] },
            last_used_idx: 0,
        }
    }

    pub fn add_avail(&mut self, desc_idx: u16) -> bool {
        if self.avail.idx == 0xffff {
            return false;
        }
        let ring_idx = (self.avail.idx as usize) % (self.queue_size as usize);
        self.avail.ring[ring_idx] = desc_idx;
        self.avail.idx += 1;
        true
    }

    pub fn get_used(&mut self) -> Option<UsedElem> {
        if self.last_used_idx == self.used.idx {
            return None;
        }
        let ring_idx = (self.last_used_idx as usize) % (self.queue_size as usize);
        let elem = self.used.ring[ring_idx];
        self.last_used_idx += 1;
        Some(elem)
    }
}

#[cfg(not(feature = "verus"))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtqueue_new() {
        let vq = Virtqueue::new(16);
        assert_eq!(vq.queue_size, 16);
        assert_eq!(vq.descriptors.len(), 16);
        assert_eq!(vq.avail.ring.len(), 16);
        assert_eq!(vq.used.ring.len(), 16);
        assert_eq!(vq.avail.idx, 0);
        assert_eq!(vq.used.idx, 0);
        assert_eq!(vq.last_used_idx, 0);
    }

    #[test]
    fn test_virtqueue_add_avail() {
        let mut vq = Virtqueue::new(4);
        assert!(vq.add_avail(1));
        assert!(vq.add_avail(2));
        assert_eq!(vq.avail.idx, 2);
        assert_eq!(vq.avail.ring[0], 1);
        assert_eq!(vq.avail.ring[1], 2);

        // Wrap around test
        assert!(vq.add_avail(3));
        assert!(vq.add_avail(4));
        assert!(vq.add_avail(5)); // This will be at index 0 (4 % 4 = 0)
        assert_eq!(vq.avail.idx, 5);
        assert_eq!(vq.avail.ring[0], 5);
        assert_eq!(vq.avail.ring[1], 2);
    }

    #[test]
    fn test_virtqueue_get_used() {
        let mut vq = Virtqueue::new(4);
        assert_eq!(vq.get_used(), None);

        // Simulate device writing to used ring
        vq.used.ring[0] = UsedElem { id: 1, len: 100 };
        vq.used.idx = 1;

        let elem = vq.get_used();
        assert_eq!(elem, Some(UsedElem { id: 1, len: 100 }));
        assert_eq!(vq.last_used_idx, 1);

        assert_eq!(vq.get_used(), None);
    }
}
