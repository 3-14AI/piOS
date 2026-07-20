#![allow(dead_code)]

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
    }

    impl NvmeDriver {
        pub fn new(capacity: u64, queue_capacity: u16) -> (d: Self)
            requires queue_capacity > 0
            ensures
                d.capacity == capacity,
                d.initialized == true,
                d.sub_queue.capacity == queue_capacity,
                d.cpl_queue.capacity == queue_capacity,
                d.sub_queue.head < queue_capacity,
                d.sub_queue.tail < queue_capacity,
                d.cpl_queue.head < queue_capacity,
                d.cpl_queue.tail < queue_capacity
        {
            NvmeDriver {
                capacity,
                initialized: true,
                sub_queue: NvmeQueue::new(queue_capacity),
                cpl_queue: NvmeQueue::new(queue_capacity),
            }
        }

        #[verifier::external_body]
        pub fn init_device(&mut self, _base_addr: usize) -> (success: bool)
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
pub struct NvmeDriver {
    pub capacity: u64,
    pub initialized: bool,
    pub sub_queue: NvmeQueue,
    pub cpl_queue: NvmeQueue,
}

#[cfg(not(feature = "verus"))]
impl NvmeDriver {
    pub fn new(capacity: u64, queue_capacity: u16) -> Self {
        NvmeDriver {
            capacity,
            initialized: true,
            sub_queue: NvmeQueue::new(queue_capacity),
            cpl_queue: NvmeQueue::new(queue_capacity),
        }
    }

    pub fn init_device(&mut self, base_addr: usize) -> bool {
        // Mocking hardware initialization logic with MMIO using raw pointers
        unsafe {
            let ptr = base_addr as *mut u32;
            let val = core::ptr::read_volatile(ptr);
            core::ptr::write_volatile(ptr, val | 1);
        }
        true
    }

    pub fn read_sector(&mut self, sector: u64, buffer_addr: usize) -> bool {
        if sector >= self.capacity {
            return false;
        }

        // Mocking DMA read to buffer
        unsafe {
            let ptr = buffer_addr as *mut u8;
            core::ptr::write_volatile(ptr, 0x55);
        }

        // Enqueue command and completion
        let _ = self.sub_queue.enqueue();
        let _ = self.cpl_queue.enqueue();

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
        let mut drv = NvmeDriver::new(1024, 16);
        assert_eq!(drv.capacity, 1024);
        assert!(drv.initialized);

        let mut mmio_mock = [0u32; 1024];
        let base_addr = mmio_mock.as_mut_ptr() as usize;

        assert_eq!(drv.init_device(base_addr), true);
        assert_eq!(mmio_mock[0], 1);

        let mut buffer = [0u8; 512];
        let buffer_addr = buffer.as_mut_ptr() as usize;
        assert_eq!(drv.read_sector(50, buffer_addr), true);
        assert_eq!(buffer[0], 0x55);

        assert_eq!(drv.read_sector(2048, buffer_addr), false);
    }

    #[test]
    fn test_block_device_trait() {
        let mut d = NvmeDriver::new(100, 16);
        let mut buffer = [0u8; 512];
        let buffer_addr = buffer.as_mut_ptr() as usize;

        // Use through trait
        let bd: &mut dyn BlockDevice = &mut d;
        assert_eq!(bd.read_sector(50, buffer_addr), true);
        assert_eq!(buffer[0], 0x55);
    }
}
