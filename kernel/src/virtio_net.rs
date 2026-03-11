#![allow(unused_imports)]

#[cfg(feature = "verus")]
use vstd::prelude::*;

#[cfg(feature = "verus")]
use crate::virtio_blk::{UsedElem, Virtqueue};

#[cfg(feature = "verus")]
verus! {
    /// VirtIO-Net Driver
    pub struct VirtioNetDriver {
        pub tx_queue: Virtqueue,
        pub rx_queue: Virtqueue,
        pub mac_address: [u8; 6],
        pub unacked_tx_packets: u32,
    }

    impl VirtioNetDriver {
        pub closed spec fn is_valid(&self) -> bool {
            self.unacked_tx_packets as int == self.tx_queue.avail.idx as int - self.tx_queue.last_used_idx as int &&
            self.tx_queue.avail.idx as int >= self.tx_queue.last_used_idx as int
        }

        pub fn new(size: u16, mac_address: [u8; 6]) -> (d: Self)
            requires size > 0
            ensures
                d.tx_queue.queue_size == size,
                d.rx_queue.queue_size == size,
                d.unacked_tx_packets == 0,
                d.is_valid()
        {
            VirtioNetDriver {
                tx_queue: Virtqueue::new(size),
                rx_queue: Virtqueue::new(size),
                mac_address,
                unacked_tx_packets: 0,
            }
        }

        pub fn send_packet(&mut self, desc_idx: u16) -> (success: bool)
            requires
                old(self).tx_queue.queue_size > 0,
                old(self).tx_queue.avail.ring.len() == old(self).tx_queue.queue_size as int,
                desc_idx < old(self).tx_queue.queue_size,
                old(self).unacked_tx_packets < 0xfffe,
                old(self).tx_queue.avail.idx < 0xffff,
                old(self).is_valid()
            ensures
                self.tx_queue.queue_size == old(self).tx_queue.queue_size,
                self.tx_queue.avail.ring.len() == old(self).tx_queue.avail.ring.len(),
                self.tx_queue.avail.ring.len() == self.tx_queue.queue_size as int,
                success ==> self.unacked_tx_packets == (old(self).unacked_tx_packets + 1),
                !success ==> self.unacked_tx_packets == old(self).unacked_tx_packets,
                self.rx_queue == old(self).rx_queue,
                self.mac_address == old(self).mac_address,
                self.tx_queue.used == old(self).tx_queue.used,
                self.tx_queue.last_used_idx == old(self).tx_queue.last_used_idx,
                self.tx_queue.descriptors == old(self).tx_queue.descriptors,
                success ==> self.tx_queue.avail.idx == (old(self).tx_queue.avail.idx + 1),
                !success ==> self.tx_queue.avail.idx == old(self).tx_queue.avail.idx,
                self.is_valid()
        {
            let ok = self.tx_queue.add_avail(desc_idx);
            if ok {
                self.unacked_tx_packets = self.unacked_tx_packets + 1;
                true
            } else {
                false
            }
        }

        pub fn process_tx_used(&mut self) -> (res: Option<UsedElem>)
            requires
                old(self).tx_queue.queue_size > 0,
                old(self).tx_queue.used.ring.len() == old(self).tx_queue.queue_size as int,
                old(self).is_valid()
            ensures
                self.tx_queue.queue_size == old(self).tx_queue.queue_size,
                self.tx_queue.used.ring.len() == old(self).tx_queue.used.ring.len(),
                self.tx_queue.used.ring.len() == self.tx_queue.queue_size as int,
                match res {
                    Some(_) => {
                        self.unacked_tx_packets == (old(self).unacked_tx_packets - 1) as u32
                    },
                    None => self.unacked_tx_packets == old(self).unacked_tx_packets
                },
                self.rx_queue == old(self).rx_queue,
                self.mac_address == old(self).mac_address,
                self.tx_queue.avail == old(self).tx_queue.avail,
                self.tx_queue.descriptors == old(self).tx_queue.descriptors,
                self.tx_queue.used == old(self).tx_queue.used,
                match res {
                    Some(_) => self.tx_queue.last_used_idx == (old(self).tx_queue.last_used_idx + 1) && old(self).tx_queue.last_used_idx < 0xffff,
                    None => self.tx_queue.last_used_idx == old(self).tx_queue.last_used_idx
                },
                self.is_valid()
        {
            if self.unacked_tx_packets > 0 {
                let used = self.tx_queue.get_used();
                match used {
                    Some(elem) => {
                        self.unacked_tx_packets = self.unacked_tx_packets - 1;
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
pub struct VirtioNetDriver {
    pub tx_queue: Virtqueue,
    pub rx_queue: Virtqueue,
    pub mac_address: [u8; 6],
    pub unacked_tx_packets: u32,
}

#[cfg(not(feature = "verus"))]
impl VirtioNetDriver {
    pub fn new(size: u16, mac_address: [u8; 6]) -> Self {
        assert!(size > 0);
        VirtioNetDriver {
            tx_queue: Virtqueue::new(size),
            rx_queue: Virtqueue::new(size),
            mac_address,
            unacked_tx_packets: 0,
        }
    }

    pub fn send_packet(&mut self, desc_idx: u16) -> bool {
        if self.unacked_tx_packets == 0xffff {
            return false;
        }
        let ok = self.tx_queue.add_avail(desc_idx);
        if ok {
            self.unacked_tx_packets += 1;
            true
        } else {
            false
        }
    }

    pub fn process_tx_used(&mut self) -> Option<UsedElem> {
        let used = self.tx_queue.get_used();
        if used.is_some() && self.unacked_tx_packets > 0 {
            self.unacked_tx_packets -= 1;
        }
        used
    }
}

#[cfg(not(feature = "verus"))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtio_net_driver() {
        let mut drv = VirtioNetDriver::new(4, [0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
        assert_eq!(drv.mac_address, [0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
        assert_eq!(drv.unacked_tx_packets, 0);

        // Send a packet
        assert_eq!(drv.send_packet(1), true);
        assert_eq!(drv.unacked_tx_packets, 1);
        assert_eq!(drv.tx_queue.avail.idx, 1);

        // Process used
        drv.tx_queue.used.ring[0] = UsedElem { id: 1, len: 100 };
        drv.tx_queue.used.idx = 1;

        let used = drv.process_tx_used();
        assert_eq!(used, Some(UsedElem { id: 1, len: 100 }));
        assert_eq!(drv.unacked_tx_packets, 0);

        // No more used
        let no_used = drv.process_tx_used();
        assert_eq!(no_used, None);
        assert_eq!(drv.unacked_tx_packets, 0);
    }
}
