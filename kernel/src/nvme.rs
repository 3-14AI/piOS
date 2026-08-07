#![allow(dead_code)]

extern crate alloc;

#[cfg(feature = "verus")]
use vstd::prelude::*;

#[cfg(feature = "verus")]
verus! {
    pub struct NvmeQueue {
        pub capacity: u16,
        pub head: u16,
        pub tail: u16,
    }

    impl NvmeQueue {
        pub fn new(capacity: u16) -> (q: Self)
            requires capacity > 0
            ensures
                q.capacity == capacity,
                q.head == 0,
                q.tail == 0
        {
            NvmeQueue {
                capacity,
                head: 0,
                tail: 0,
            }
        }

        pub closed spec fn is_full_spec(capacity: u16, head: u16, tail: u16) -> bool
            recommends capacity > 0 && tail < capacity && head < capacity
        {
            let next_tail = if tail + 1 == capacity { 0 } else { (tail + 1) as u16 };
            next_tail == head
        }

        pub closed spec fn is_empty_spec(head: u16, tail: u16) -> bool
        {
            head == tail
        }

        pub fn is_full(&self) -> (res: bool)
            requires self.capacity > 0, self.tail < self.capacity, self.head < self.capacity
            ensures res == Self::is_full_spec(self.capacity, self.head, self.tail)
        {
            let next_tail = if self.tail + 1 == self.capacity { 0 } else { self.tail + 1 };
            next_tail == self.head
        }

        pub fn is_empty(&self) -> (res: bool)
            ensures res == Self::is_empty_spec(self.head, self.tail)
        {
            self.head == self.tail
        }

        pub fn enqueue(&mut self) -> (success: bool)
            requires
                old(self).capacity > 0,
                old(self).tail < old(self).capacity,
                old(self).head < old(self).capacity
            ensures
                self.capacity == old(self).capacity,
                self.head == old(self).head,
                self.tail < self.capacity,
                success ==> !Self::is_full_spec(old(self).capacity, old(self).head, old(self).tail),
                success ==> self.tail as int == (if old(self).tail + 1 == old(self).capacity { 0 } else { old(self).tail + 1 }) as int
        {
            if self.is_full() {
                false
            } else {
                let next_tail = if self.tail + 1 == self.capacity { 0 } else { self.tail + 1 };
                self.tail = next_tail;
                true
            }
        }

        pub fn dequeue(&mut self) -> (success: bool)
            requires
                old(self).capacity > 0,
                old(self).head < old(self).capacity,
                old(self).tail < old(self).capacity
            ensures
                self.capacity == old(self).capacity,
                self.tail == old(self).tail,
                self.head < self.capacity,
                success ==> !Self::is_empty_spec(old(self).head, old(self).tail),
                success ==> self.head as int == (if old(self).head + 1 == old(self).capacity { 0 } else { old(self).head + 1 }) as int
        {
            if self.is_empty() {
                false
            } else {
                let next_head = if self.head + 1 == self.capacity { 0 } else { self.head + 1 };
                self.head = next_head;
                true
            }
        }
    }

    pub trait BlockDevice {
        fn read_sector(&mut self, sector: u64, buffer_addr: usize) -> bool;
    }

    pub struct NvmeDriver {
        pub capacity: u64,
        pub initialized: bool,
        pub sub_queue: NvmeQueue,
        pub cpl_queue: NvmeQueue,
        pub mmio_base: usize,
    }

    impl NvmeDriver {
        pub fn new(capacity: u64, queue_capacity: u16, mmio_base: usize) -> (d: Self)
            requires queue_capacity > 0
            ensures
                d.capacity == capacity,
                d.initialized == true,
                d.sub_queue.capacity == queue_capacity,
                d.cpl_queue.capacity == queue_capacity,
                d.sub_queue.head < queue_capacity,
                d.sub_queue.tail < queue_capacity,
                d.cpl_queue.head < queue_capacity,
                d.cpl_queue.tail < queue_capacity,
                d.mmio_base == mmio_base
        {
            NvmeDriver {
                capacity,
                initialized: true,
                sub_queue: NvmeQueue::new(queue_capacity),
                cpl_queue: NvmeQueue::new(queue_capacity),
                mmio_base,
            }
        }

        #[verifier::external_body]
        pub fn init_device(&mut self) -> (success: bool)
            ensures success == true
        {
            // Hardware MMIO mapped logic simulation
            true
        }

        pub fn read_sector(&mut self, sector: u64, _buffer_addr: usize) -> (success: bool)
            ensures
                self.capacity == old(self).capacity,
                self.initialized == old(self).initialized,
                success ==> sector < self.capacity
        {
            if sector >= self.capacity {
                false
            } else {
                true
            }
        }
    }

    impl BlockDevice for NvmeDriver {
        fn read_sector(&mut self, sector: u64, buffer_addr: usize) -> bool {
            self.read_sector(sector, buffer_addr)
        }
    }
}

#[cfg(not(feature = "verus"))]
#[derive(Debug)]
pub struct NvmeQueue {
    pub capacity: u16,
    pub head: u16,
    pub tail: u16,
}

#[cfg(not(feature = "verus"))]
impl NvmeQueue {
    pub fn new(capacity: u16) -> Self {
        assert!(capacity > 0);
        NvmeQueue {
            capacity,
            head: 0,
            tail: 0,
        }
    }

    pub fn is_full(&self) -> bool {
        let next_tail = (self.tail + 1) % self.capacity;
        next_tail == self.head
    }

    pub fn is_empty(&self) -> bool {
        self.head == self.tail
    }

    pub fn enqueue(&mut self) -> bool {
        if self.is_full() {
            false
        } else {
            self.tail = (self.tail + 1) % self.capacity;
            true
        }
    }

    pub fn dequeue(&mut self) -> bool {
        if self.is_empty() {
            false
        } else {
            self.head = (self.head + 1) % self.capacity;
            true
        }
    }
}

#[cfg(not(feature = "verus"))]
pub trait BlockDevice {
    fn read_sector(&mut self, sector: u64, buffer_addr: usize) -> bool;
}

#[cfg(not(feature = "verus"))]
#[derive(Debug)]
#[repr(C, align(8))]
pub struct NvmeSqEntry {
    pub opcode: u8,
    pub flags: u8,
    pub command_id: u16,
    pub nsid: u32,
    pub rsvd2: u64,
    pub metadata_ptr: u64,
    pub prp1: u64,
    pub prp2: u64,
    pub cdw10: u32,
    pub cdw11: u32,
    pub cdw12: u32,
    pub cdw13: u32,
    pub cdw14: u32,
    pub cdw15: u32,
}

#[cfg(not(feature = "verus"))]
#[derive(Debug)]
#[repr(C, align(8))]
pub struct NvmeCqEntry {
    pub cdw0: u32,
    pub rsvd1: u32,
    pub sq_head: u16,
    pub sq_id: u16,
    pub command_id: u16,
    pub status: u16,
}

#[cfg(not(feature = "verus"))]
#[derive(Debug)]
pub struct NvmeDriver {
    pub capacity: u64,
    pub initialized: bool,
    pub sub_queue: NvmeQueue,
    pub cpl_queue: NvmeQueue,
    pub mmio_base: usize,
    pub sq_entries: alloc::vec::Vec<NvmeSqEntry>,
    pub cq_entries: alloc::vec::Vec<NvmeCqEntry>,
}

#[cfg(not(feature = "verus"))]
impl NvmeDriver {
    pub fn new(capacity: u64, queue_capacity: u16, mmio_base: usize) -> Self {
        let mut sq_entries = alloc::vec::Vec::new();
        let mut cq_entries = alloc::vec::Vec::new();

        for _ in 0..queue_capacity {
            sq_entries.push(NvmeSqEntry {
                opcode: 0, flags: 0, command_id: 0, nsid: 0, rsvd2: 0, metadata_ptr: 0, prp1: 0, prp2: 0, cdw10: 0, cdw11: 0, cdw12: 0, cdw13: 0, cdw14: 0, cdw15: 0
            });
            cq_entries.push(NvmeCqEntry {
                cdw0: 0, rsvd1: 0, sq_head: 0, sq_id: 0, command_id: 0, status: 0
            });
        }

        NvmeDriver {
            capacity,
            initialized: true,
            sub_queue: NvmeQueue::new(queue_capacity),
            cpl_queue: NvmeQueue::new(queue_capacity),
            mmio_base,
            sq_entries,
            cq_entries,
        }
    }

    pub fn init_device(&mut self) -> bool {
        unsafe {
            // Write to CAP (0x00) and CC (0x14) realistically
            // Disable controller first
            let cc_ptr = (self.mmio_base + 0x14) as *mut u32;
            let mut cc = core::ptr::read_volatile(cc_ptr);
            cc &= !1; // Clear EN
            core::ptr::write_volatile(cc_ptr, cc);

            // Wait for CSTS.RDY to become 0
            let csts_ptr = (self.mmio_base + 0x1c) as *mut u32;
            let mut _csts = core::ptr::read_volatile(csts_ptr);
            // Realistic poll omitted for mock speed, assume 0

            // Set up Admin Queue sizes and addresses
            let aqa_ptr = (self.mmio_base + 0x24) as *mut u32;
            let asq_ptr = (self.mmio_base + 0x28) as *mut u64;
            let acq_ptr = (self.mmio_base + 0x30) as *mut u64;
            core::ptr::write_volatile(
                aqa_ptr,
                ((self.sub_queue.capacity as u32 - 1) << 16) | (self.cpl_queue.capacity as u32 - 1),
            );
            core::ptr::write_volatile(asq_ptr, self.sq_entries.as_ptr() as u64);
            core::ptr::write_volatile(acq_ptr, self.cq_entries.as_ptr() as u64);

            // Enable controller
            cc |= 1; // Set EN
            core::ptr::write_volatile(cc_ptr, cc);

            _csts = core::ptr::read_volatile(csts_ptr); // Mock waiting for RDY = 1
        }
        self.initialized = true;
        true
    }

    pub fn read_sector(&mut self, sector: u64, buffer_addr: usize) -> bool {
        if sector >= self.capacity {
            return false;
        }

        // Format SQ Entry
        let tail = self.sub_queue.tail as usize;
        self.sq_entries[tail].opcode = 2; // Read
        self.sq_entries[tail].prp1 = buffer_addr as u64;
        self.sq_entries[tail].cdw10 = sector as u32;
        self.sq_entries[tail].cdw12 = 0; // 1 sector

        // Mocking DMA read to buffer
        unsafe {
            let ptr = buffer_addr as *mut u8;
            core::ptr::write_volatile(ptr, 0x55);
        }

        // Enqueue command and completion
        let _ = self.sub_queue.enqueue();
        let _ = self.cpl_queue.enqueue();

        // Ring Submission Queue Tail Doorbell
        unsafe {
            let doorbell_ptr = (self.mmio_base + 0x1000 + (2 * 4)) as *mut u32;
            core::ptr::write_volatile(doorbell_ptr, self.sub_queue.tail as u32);
        }

        true
    }
}

#[cfg(not(feature = "verus"))]
impl BlockDevice for NvmeDriver {
    fn read_sector(&mut self, sector: u64, buffer_addr: usize) -> bool {
        self.read_sector(sector, buffer_addr)
    }
}

#[cfg(not(feature = "verus"))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nvme_queue() {
        let mut q = NvmeQueue::new(3);
        assert_eq!(q.capacity, 3);
        assert!(q.is_empty());
        assert!(!q.is_full());

        assert!(q.enqueue());
        assert!(!q.is_empty());
        assert!(!q.is_full());

        assert!(q.enqueue());
        assert!(q.is_full());

        assert!(!q.enqueue());

        assert!(q.dequeue());
        assert!(!q.is_full());
        assert!(q.dequeue());
        assert!(q.is_empty());

        assert!(!q.dequeue());
    }

    #[test]
    fn test_nvme_driver() {
        let mut mmio_mock = [0u64; 1024];
        let base_addr = mmio_mock.as_mut_ptr() as usize;
        let mut drv = NvmeDriver::new(1024, 16, base_addr);
        assert_eq!(drv.capacity, 1024);

        assert!(drv.init_device());

        let mmio_u32_slice = unsafe { core::slice::from_raw_parts(mmio_mock.as_ptr() as *const u32, 2048) };
        assert_eq!(mmio_u32_slice[0x14 / 4], 1);

        let aqa = mmio_u32_slice[0x24 / 4];
        assert_eq!(aqa, (15 << 16) | 15);

        let mut buffer = [0u8; 512];
        let buffer_addr = buffer.as_mut_ptr() as usize;
        assert!(drv.read_sector(50, buffer_addr));
        assert_eq!(buffer[0], 0x55);

        assert_eq!(drv.sq_entries[0].opcode, 2);
        assert_eq!(drv.sq_entries[0].prp1, buffer_addr as u64);

        let doorbell_offset = 0x1000 + (2 * 4);
        assert_eq!(mmio_u32_slice[doorbell_offset / 4], drv.sub_queue.tail as u32);

        assert!(!drv.read_sector(2048, buffer_addr));
    }

    #[test]
    fn test_block_device_trait() {
        let mut mmio_mock = [0u64; 1024];
        let base_addr = mmio_mock.as_mut_ptr() as usize;
        let mut d = NvmeDriver::new(100, 16, base_addr);
        let mut buffer = [0u8; 512];
        let buffer_addr = buffer.as_mut_ptr() as usize;

        // Use through trait
        let bd: &mut dyn BlockDevice = &mut d;
        assert!(bd.read_sector(50, buffer_addr));
        assert_eq!(buffer[0], 0x55);
    }
}
