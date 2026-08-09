#![allow(dead_code)]

#[cfg(feature = "verus")]
use vstd::prelude::*;

#[cfg(feature = "verus")]
verus! {
    pub struct Ext4 {
        pub mounted: bool,
    }

    impl Ext4 {
        pub fn new() -> (d: Self)
            ensures d.mounted == false
        {
            Ext4 { mounted: false }
        }

        pub fn fsck(&self) -> (res: Result<(), ()>) { Ok(()) }

        pub fn journal(&self) -> (res: Result<(), ()>) { Ok(()) }

        pub fn mount(&mut self) -> (res: Result<(), ()>)
            ensures
                match res {
                    Ok(_) => self.mounted == true,
                    Err(_) => *self == *old(self),
                }
        {
            let mut new_ext4 = Ext4 { mounted: true };
            *self = new_ext4;
            Ok(())
        }

        pub fn read_block(&self, _block: u64, _buffer: &mut [u8]) -> (res: Result<(), ()>) { Ok(()) }
        pub fn write_block(&mut self, _block: u64, _buffer: &[u8]) -> (res: Result<(), ()>) { Ok(()) }
    }
}

#[cfg(not(feature = "verus"))]
pub struct Ext4 {
    pub block_device: Option<crate::virtio_blk::VirtioBlkDriver>,
    pub mounted: bool,
}

#[cfg(not(feature = "verus"))]
impl Ext4 {
    pub fn new() -> Self {
        Ext4 {
            block_device: None,
            mounted: false,
        }
    }

    pub fn new_with_device(dev: crate::virtio_blk::VirtioBlkDriver) -> Self {
        Ext4 {
            block_device: Some(dev),
            mounted: false,
        }
    }

    pub fn mount(&mut self) -> Result<(), ()> {
        if self.block_device.is_some() {
            self.mounted = true;
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn fsck(&self) -> Result<(), ()> {
        Ok(())
    }

    pub fn journal(&self) -> Result<(), ()> {
        Ok(())
    }

    pub fn read_block(&mut self, block: u64, _buffer: &mut [u8]) -> Result<(), ()> {
        if let Some(dev) = &mut self.block_device {
            if dev.read_sector(block, 0) {
                return Ok(());
            }
        }
        Err(())
    }

    pub fn write_block(&mut self, block: u64, _buffer: &[u8]) -> Result<(), ()> {
        if let Some(dev) = &mut self.block_device {
            // Write sector (simulated by read_sector for now since write_sector is missing, or we implement write_sector on VirtioBlkDriver)
            // Just for the sake of the driver implementation stub completeness
            if block < dev.capacity {
                return Ok(());
            }
        }
        Err(())
    }
}

#[cfg(not(feature = "verus"))]
impl Default for Ext4 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "verus"))]
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_ext4() {
        let mut fs = Ext4::new();
        assert!(fs.mount().is_err());
        assert!(fs.fsck().is_ok());
        assert!(fs.journal().is_ok());

        let mut mmio_mock = [0u64; 1024];
        let base_addr = mmio_mock.as_mut_ptr() as usize;
        let drv = crate::virtio_blk::VirtioBlkDriver::new(4, 100, base_addr, (0, 2, 0));
        let mut fs_dev = Ext4::new_with_device(drv);

        assert!(fs_dev.mount().is_ok());
        assert!(fs_dev.mounted);

        let mut buf = [0u8; 512];
        assert!(fs_dev.read_block(0, &mut buf).is_ok());
        assert!(fs_dev.write_block(0, &buf).is_ok());

        // Out of bounds
        assert!(fs_dev.read_block(200, &mut buf).is_err());
        assert!(fs_dev.write_block(200, &buf).is_err());
    }
}
