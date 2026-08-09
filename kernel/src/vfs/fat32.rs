#![allow(dead_code)]

#[cfg(feature = "verus")]
use vstd::prelude::*;

#[cfg(feature = "verus")]
verus! {
    pub struct Fat32 {
        pub mounted: bool,
    }

    impl Fat32 {
        pub fn new() -> (d: Self)
            ensures d.mounted == false
        {
            Fat32 { mounted: false }
        }

        pub fn fsck(&self) -> (res: Result<(), ()>) { Ok(()) }

        pub fn mount(&mut self) -> (res: Result<(), ()>)
            ensures
                match res {
                    Ok(_) => self.mounted == true,
                    Err(_) => *self == *old(self),
                }
        {
            let mut new_fat = Fat32 { mounted: true };
            *self = new_fat;
            Ok(())
        }

        pub fn read_block(&self, _block: u64, _buffer: &mut [u8]) -> (res: Result<(), ()>) { Ok(()) }
        pub fn write_block(&mut self, _block: u64, _buffer: &[u8]) -> (res: Result<(), ()>) { Ok(()) }
    }
}

#[cfg(not(feature = "verus"))]
pub struct Fat32 {
    pub block_device: Option<crate::virtio_blk::VirtioBlkDriver>,
    pub mounted: bool,
}

#[cfg(not(feature = "verus"))]
impl Fat32 {
    pub fn new() -> Self {
        Fat32 {
            block_device: None,
            mounted: false,
        }
    }

    pub fn new_with_device(dev: crate::virtio_blk::VirtioBlkDriver) -> Self {
        Fat32 {
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
            if block < dev.capacity {
                return Ok(());
            }
        }
        Err(())
    }
}

#[cfg(not(feature = "verus"))]
impl Default for Fat32 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "verus"))]
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_fat32() {
        let mut fs = Fat32::new();
        assert!(fs.mount().is_err());
        assert!(fs.fsck().is_ok());

        let mut mmio_mock = [0u64; 1024];
        let base_addr = mmio_mock.as_mut_ptr() as usize;
        let drv = crate::virtio_blk::VirtioBlkDriver::new(4, 100, base_addr, (0, 2, 0));
        let mut fs_dev = Fat32::new_with_device(drv);

        assert!(fs_dev.mount().is_ok());
        assert!(fs_dev.mounted);

        let mut buf = [0u8; 512];
        assert!(fs_dev.read_block(0, &mut buf).is_ok());
        assert!(fs_dev.write_block(0, &buf).is_ok());

        assert!(fs_dev.read_block(200, &mut buf).is_err());
        assert!(fs_dev.write_block(200, &buf).is_err());
    }
}
