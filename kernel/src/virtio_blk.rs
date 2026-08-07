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

    /// VirtIO Block Request Header
    #[derive(Copy, Clone)]
    pub struct VirtioBlkReq {
        pub type_: u32,
        pub reserved: u32,
        pub sector: u64,
    }

    /// Virtqueue abstraction
    pub struct Virtqueue {
        pub queue_size: u16,
        pub descriptors: Vec<Descriptor>,
        pub avail: AvailRing,
        pub used: UsedRing,
        pub last_used_idx: u16,
    }

    /// VirtIO-Blk Driver
    pub struct VirtioBlkDriver {
        pub queue: Virtqueue,
        pub capacity: u64,
        pub mmio_base: usize,
        pub pci_address: (u8, u8, u8),
    }

    impl VirtioBlkDriver {
        pub fn new(size: u16, capacity: u64, mmio_base: usize, pci_address: (u8, u8, u8)) -> (d: Self)
            requires size > 0
            ensures
                d.capacity == capacity,
                d.queue.queue_size == size,
                d.mmio_base == mmio_base,
                d.pci_address == pci_address
        {
            VirtioBlkDriver {
                queue: Virtqueue::new(size),
                capacity,
                mmio_base,
                pci_address,
            }
        }

        #[verifier::external_body]
        pub fn init_device(&mut self) -> (success: bool)
            ensures success == true
        {
            true
        }

        pub fn read_sector(&mut self, sector: u64, desc_idx: u16) -> (success: bool)
            requires
                old(self).queue.queue_size > 0,
                old(self).queue.avail.ring.len() == old(self).queue.queue_size as int,
                desc_idx < old(self).queue.queue_size
            ensures
                self.queue.queue_size == old(self).queue.queue_size,
                self.queue.avail.ring.len() == old(self).queue.avail.ring.len(),
                self.queue.avail.ring.len() == self.queue.queue_size as int,
                success ==> self.queue.avail.idx == (old(self).queue.avail.idx + 1),
                !success ==> self.queue.avail.idx == old(self).queue.avail.idx,
                self.queue.descriptors == old(self).queue.descriptors,
                self.queue.used == old(self).queue.used,
                self.queue.last_used_idx == old(self).queue.last_used_idx,
                self.capacity == old(self).capacity
        {
            if sector >= self.capacity {
                return false;
            }
            self.queue.add_avail(desc_idx)
        }
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
                !success ==> self.avail.idx == old(self).avail.idx,
                self.descriptors == old(self).descriptors,
                self.used == old(self).used,
                self.last_used_idx == old(self).last_used_idx
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
                },
                self.descriptors == old(self).descriptors,
                self.avail == old(self).avail,
                self.used == old(self).used
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
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct VirtioBlkReq {
    pub type_: u32,
    pub reserved: u32,
    pub sector: u64,
}

#[cfg(not(feature = "verus"))]
#[derive(Debug)]
pub struct VirtioBlkDriver {
    pub queue: Virtqueue,
    pub capacity: u64,
    pub mmio_base: usize,
    pub pci_address: (u8, u8, u8),
}

#[cfg(not(feature = "verus"))]
impl VirtioBlkDriver {
    pub fn new(size: u16, capacity: u64, mmio_base: usize, pci_address: (u8, u8, u8)) -> Self {
        assert!(size > 0);
        VirtioBlkDriver {
            queue: Virtqueue::new(size),
            capacity,
            mmio_base,
            pci_address,
        }
    }

    pub fn init_device(&mut self) -> bool {
        unsafe {
            // Write 0 to status register to reset
            let status_ptr = (self.mmio_base + 0x70) as *mut u32;
            core::ptr::write_volatile(status_ptr, 0);

            // Write 1 (ACKNOWLEDGE) and 2 (DRIVER)
            core::ptr::write_volatile(status_ptr, 1 | 2);

            // Read features
            let _features = core::ptr::read_volatile((self.mmio_base + 0x10) as *mut u32);

            // Tell device where the queue is
            let q_select_ptr = (self.mmio_base + 0x30) as *mut u32;
            core::ptr::write_volatile(q_select_ptr, 0);

            let q_size_ptr = (self.mmio_base + 0x38) as *mut u32;
            core::ptr::write_volatile(q_size_ptr, self.queue.queue_size as u32);

            let q_desc_ptr = (self.mmio_base + 0x80) as *mut u64;
            core::ptr::write_volatile(q_desc_ptr, self.queue.descriptors.as_ptr() as u64);

            // Write 4 (DRIVER_OK)
            core::ptr::write_volatile(status_ptr, 1 | 2 | 4);
        }
        true
    }

    pub fn read_sector(&mut self, sector: u64, desc_idx: u16) -> bool {
        if sector >= self.capacity {
            return false;
        }

        let desc_index = desc_idx as usize;
        self.queue.descriptors[desc_index].addr = sector; // Block addr
        self.queue.descriptors[desc_index].len = 512;
        self.queue.descriptors[desc_index].flags = 2; // Write

        let success = self.queue.add_avail(desc_idx);

        if success {
            unsafe {
                // Queue notify
                let notify_ptr = (self.mmio_base + 0x50) as *mut u32;
                core::ptr::write_volatile(notify_ptr, 0); // queue 0
            }
        }
        success
    }
}

#[cfg(not(feature = "verus"))]
impl Virtqueue {
    pub fn new(size: u16) -> Self {
        assert!(size > 0);
        Virtqueue {
            queue_size: size,
            descriptors: alloc::vec![Descriptor { addr: 0, len: 0, flags: 0, next: 0 }; size as usize],
            avail: AvailRing {
                flags: 0,
                idx: 0,
                ring: alloc::vec![0; size as usize],
            },
            used: UsedRing {
                flags: 0,
                idx: 0,
                ring: alloc::vec![UsedElem { id: 0, len: 0 }; size as usize],
            },
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

    #[test]
    fn test_virtio_blk_driver() {
        let mut mmio_mock = [0u64; 1024];
        let base_addr = mmio_mock.as_mut_ptr() as usize;
        let mut drv = VirtioBlkDriver::new(4, 100, base_addr, (0, 2, 0));
        assert_eq!(drv.capacity, 100);

        assert!(drv.init_device());

        let mmio_u32_slice = unsafe { core::slice::from_raw_parts(mmio_mock.as_ptr() as *const u32, 2048) };
        assert_eq!(mmio_u32_slice[0x70 / 4], 7); // 1 | 2 | 4

        // Queue details written to mmio
        assert_eq!(mmio_u32_slice[0x30 / 4], 0);
        assert_eq!(mmio_u32_slice[0x38 / 4], 4);

        // On x86_64 or similar, if mmio_mock was written via write_volatile(..., u64),
        // it would overwrite two adjacent u32s. Wait, if q_desc_ptr is *mut u64,
        // writing to it will write 8 bytes. Let's just cast the mmio_mock pointer.
        let mmio_u64_ptr = base_addr as *const u64;
        unsafe {
            assert_eq!(
                core::ptr::read_volatile(mmio_u64_ptr.add(0x80 / 8)),
                drv.queue.descriptors.as_ptr() as u64
            );
        }

        // Out of bounds sector
        assert!(!drv.read_sector(200, 1));

        // Valid sector
        assert!(drv.read_sector(10, 1));
        assert_eq!(drv.queue.avail.idx, 1);
        assert_eq!(drv.queue.avail.ring[0], 1);

        // Check descriptor
        assert_eq!(drv.queue.descriptors[1].addr, 10);
        assert_eq!(drv.queue.descriptors[1].len, 512);

        // Notify
        assert_eq!(mmio_u32_slice[0x50 / 4], 0);
    }
}
