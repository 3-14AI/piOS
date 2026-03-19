#![allow(unused_imports)]

#[cfg(feature = "verus")]
use vstd::prelude::*;

#[cfg(feature = "verus")]
use crate::virtio_blk::{UsedElem, Virtqueue};

#[cfg(feature = "verus")]
verus! {
    /// VirtIO-GPU Driver
    pub struct VirtioGpuDriver {
        pub control_queue: Virtqueue,
        pub cursor_queue: Virtqueue,
        pub unacked_commands: u32,
    }

    impl VirtioGpuDriver {
        pub closed spec fn is_valid(&self) -> bool {
            self.unacked_commands as int == self.control_queue.avail.idx as int - self.control_queue.last_used_idx as int &&
            self.control_queue.avail.idx as int >= self.control_queue.last_used_idx as int
        }

        pub fn new(size: u16) -> (d: Self)
            requires size > 0
            ensures
                d.control_queue.queue_size == size,
                d.cursor_queue.queue_size == size,
                d.unacked_commands == 0,
                d.is_valid()
        {
            VirtioGpuDriver {
                control_queue: Virtqueue::new(size),
                cursor_queue: Virtqueue::new(size),
                unacked_commands: 0,
            }
        }

        pub fn send_command(&mut self, desc_idx: u16) -> (success: bool)
            requires
                old(self).control_queue.queue_size > 0,
                old(self).control_queue.avail.ring.len() == old(self).control_queue.queue_size as int,
                desc_idx < old(self).control_queue.queue_size,
                old(self).unacked_commands < 0xfffe,
                old(self).control_queue.avail.idx < 0xffff,
                old(self).is_valid()
            ensures
                self.control_queue.queue_size == old(self).control_queue.queue_size,
                self.control_queue.avail.ring.len() == old(self).control_queue.avail.ring.len(),
                self.control_queue.avail.ring.len() == self.control_queue.queue_size as int,
                success ==> self.unacked_commands == (old(self).unacked_commands + 1),
                !success ==> self.unacked_commands == old(self).unacked_commands,
                self.cursor_queue == old(self).cursor_queue,
                self.control_queue.used == old(self).control_queue.used,
                self.control_queue.last_used_idx == old(self).control_queue.last_used_idx,
                self.control_queue.descriptors == old(self).control_queue.descriptors,
                success ==> self.control_queue.avail.idx == (old(self).control_queue.avail.idx + 1),
                !success ==> self.control_queue.avail.idx == old(self).control_queue.avail.idx,
                self.is_valid()
        {
            let ok = self.control_queue.add_avail(desc_idx);
            if ok {
                self.unacked_commands = self.unacked_commands + 1;
                true
            } else {
                false
            }
        }

        pub fn enqueue_2d_command(&mut self, desc_idx: u16) -> (success: bool)
            requires
                old(self).control_queue.queue_size > 0,
                old(self).control_queue.avail.ring.len() == old(self).control_queue.queue_size as int,
                desc_idx < old(self).control_queue.queue_size,
                old(self).unacked_commands < 0xfffe,
                old(self).control_queue.avail.idx < 0xffff,
                old(self).is_valid()
            ensures
                self.control_queue.queue_size == old(self).control_queue.queue_size,
                self.control_queue.avail.ring.len() == old(self).control_queue.avail.ring.len(),
                self.control_queue.avail.ring.len() == self.control_queue.queue_size as int,
                success ==> self.unacked_commands == (old(self).unacked_commands + 1),
                !success ==> self.unacked_commands == old(self).unacked_commands,
                self.cursor_queue == old(self).cursor_queue,
                self.control_queue.used == old(self).control_queue.used,
                self.control_queue.last_used_idx == old(self).control_queue.last_used_idx,
                self.control_queue.descriptors == old(self).control_queue.descriptors,
                success ==> self.control_queue.avail.idx == (old(self).control_queue.avail.idx + 1),
                !success ==> self.control_queue.avail.idx == old(self).control_queue.avail.idx,
                self.is_valid()
        {
            self.send_command(desc_idx)
        }

        pub fn enqueue_3d_command(&mut self, desc_idx: u16) -> (success: bool)
            requires
                old(self).control_queue.queue_size > 0,
                old(self).control_queue.avail.ring.len() == old(self).control_queue.queue_size as int,
                desc_idx < old(self).control_queue.queue_size,
                old(self).unacked_commands < 0xfffe,
                old(self).control_queue.avail.idx < 0xffff,
                old(self).is_valid()
            ensures
                self.control_queue.queue_size == old(self).control_queue.queue_size,
                self.control_queue.avail.ring.len() == old(self).control_queue.avail.ring.len(),
                self.control_queue.avail.ring.len() == self.control_queue.queue_size as int,
                success ==> self.unacked_commands == (old(self).unacked_commands + 1),
                !success ==> self.unacked_commands == old(self).unacked_commands,
                self.cursor_queue == old(self).cursor_queue,
                self.control_queue.used == old(self).control_queue.used,
                self.control_queue.last_used_idx == old(self).control_queue.last_used_idx,
                self.control_queue.descriptors == old(self).control_queue.descriptors,
                success ==> self.control_queue.avail.idx == (old(self).control_queue.avail.idx + 1),
                !success ==> self.control_queue.avail.idx == old(self).control_queue.avail.idx,
                self.is_valid()
        {
            self.send_command(desc_idx)
        }

        pub fn process_command_used(&mut self) -> (res: Option<UsedElem>)
            requires
                old(self).control_queue.queue_size > 0,
                old(self).control_queue.used.ring.len() == old(self).control_queue.queue_size as int,
                old(self).is_valid()
            ensures
                self.control_queue.queue_size == old(self).control_queue.queue_size,
                self.control_queue.used.ring.len() == old(self).control_queue.used.ring.len(),
                self.control_queue.used.ring.len() == self.control_queue.queue_size as int,
                match res {
                    Some(_) => {
                        self.unacked_commands == (old(self).unacked_commands - 1) as u32
                    },
                    None => self.unacked_commands == old(self).unacked_commands
                },
                self.cursor_queue == old(self).cursor_queue,
                self.control_queue.avail == old(self).control_queue.avail,
                self.control_queue.descriptors == old(self).control_queue.descriptors,
                self.control_queue.used == old(self).control_queue.used,
                match res {
                    Some(_) => self.control_queue.last_used_idx == (old(self).control_queue.last_used_idx + 1) && old(self).control_queue.last_used_idx < 0xffff,
                    None => self.control_queue.last_used_idx == old(self).control_queue.last_used_idx
                },
                self.is_valid()
        {
            if self.unacked_commands > 0 {
                let used = self.control_queue.get_used();
                match used {
                    Some(elem) => {
                        self.unacked_commands = self.unacked_commands - 1;
                        Some(elem)
                    },
                    None => None,
                }
            } else {
                None
            }
        }
    }
}

#[cfg(not(feature = "verus"))]
use crate::virtio_blk::{UsedElem, Virtqueue};

#[cfg(not(feature = "verus"))]
#[derive(Debug)]
pub struct VirtioGpuDriver {
    pub control_queue: Virtqueue,
    pub cursor_queue: Virtqueue,
    pub unacked_commands: u32,
}

#[cfg(not(feature = "verus"))]
impl VirtioGpuDriver {
    pub fn new(size: u16) -> Self {
        assert!(size > 0);
        VirtioGpuDriver {
            control_queue: Virtqueue::new(size),
            cursor_queue: Virtqueue::new(size),
            unacked_commands: 0,
        }
    }

    pub fn send_command(&mut self, desc_idx: u16) -> bool {
        if self.unacked_commands == 0xffff {
            return false;
        }
        let ok = self.control_queue.add_avail(desc_idx);
        if ok {
            self.unacked_commands += 1;
            true
        } else {
            false
        }
    }

    pub fn enqueue_2d_command(&mut self, desc_idx: u16) -> bool {
        self.send_command(desc_idx)
    }

    pub fn enqueue_3d_command(&mut self, desc_idx: u16) -> bool {
        self.send_command(desc_idx)
    }

    pub fn process_command_used(&mut self) -> Option<UsedElem> {
        let used = self.control_queue.get_used();
        if used.is_some() && self.unacked_commands > 0 {
            self.unacked_commands -= 1;
        }
        used
    }
}

#[cfg(not(feature = "verus"))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtio_gpu_driver() {
        let mut drv = VirtioGpuDriver::new(4);
        assert_eq!(drv.unacked_commands, 0);

        // Send a command
        assert_eq!(drv.send_command(1), true);
        assert_eq!(drv.unacked_commands, 1);
        assert_eq!(drv.control_queue.avail.idx, 1);

        // Process used
        drv.control_queue.used.ring[0] = UsedElem { id: 1, len: 100 };
        drv.control_queue.used.idx = 1;

        let used = drv.process_command_used();
        assert_eq!(used, Some(UsedElem { id: 1, len: 100 }));
        assert_eq!(drv.unacked_commands, 0);

        // No more used
        let no_used = drv.process_command_used();
        assert_eq!(no_used, None);
        assert_eq!(drv.unacked_commands, 0);
    }

    #[test]
    fn test_enqueue_2d_command() {
        let mut drv = VirtioGpuDriver::new(4);
        assert_eq!(drv.unacked_commands, 0);

        // Send a 2d command
        assert_eq!(drv.enqueue_2d_command(2), true);
        assert_eq!(drv.unacked_commands, 1);
        assert_eq!(drv.control_queue.avail.idx, 1);
        assert_eq!(drv.control_queue.avail.ring[0], 2);
    }

    #[test]
    fn test_enqueue_3d_command() {
        let mut drv = VirtioGpuDriver::new(4);
        assert_eq!(drv.unacked_commands, 0);

        // Send a 3d command
        assert_eq!(drv.enqueue_3d_command(3), true);
        assert_eq!(drv.unacked_commands, 1);
        assert_eq!(drv.control_queue.avail.idx, 1);
        assert_eq!(drv.control_queue.avail.ring[0], 3);
    }
}
