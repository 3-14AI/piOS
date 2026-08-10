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
#[derive(Debug, Clone, Copy)]
pub struct Ext4Superblock {
    pub inodes_count: u32,
    pub blocks_count_lo: u32,
    pub r_blocks_count_lo: u32,
    pub free_blocks_count_lo: u32,
    pub free_inodes_count: u32,
    pub first_data_block: u32,
    pub log_block_size: u32,
    pub log_cluster_size: u32,
    pub blocks_per_group: u32,
    pub clusters_per_group: u32,
    pub inodes_per_group: u32,
    pub mtime: u32,
    pub wtime: u32,
    pub mnt_count: u16,
    pub max_mnt_count: u16,
    pub magic: u16,
    pub state: u16,
    pub errors: u16,
    pub minor_rev_level: u16,
}

#[cfg(not(feature = "verus"))]
impl Ext4Superblock {
    pub fn parse(buffer: &[u8]) -> Option<Self> {
        if buffer.len() < 1024 {
            return None; // Superblock is at least 1024 bytes long
        }

        let magic = u16::from_le_bytes([buffer[56], buffer[57]]);
        if magic != 0xEF53 {
            return None; // Invalid magic number
        }

        Some(Self {
            inodes_count: u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]),
            blocks_count_lo: u32::from_le_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]),
            r_blocks_count_lo: u32::from_le_bytes([buffer[8], buffer[9], buffer[10], buffer[11]]),
            free_blocks_count_lo: u32::from_le_bytes([buffer[12], buffer[13], buffer[14], buffer[15]]),
            free_inodes_count: u32::from_le_bytes([buffer[16], buffer[17], buffer[18], buffer[19]]),
            first_data_block: u32::from_le_bytes([buffer[20], buffer[21], buffer[22], buffer[23]]),
            log_block_size: u32::from_le_bytes([buffer[24], buffer[25], buffer[26], buffer[27]]),
            log_cluster_size: u32::from_le_bytes([buffer[28], buffer[29], buffer[30], buffer[31]]),
            blocks_per_group: u32::from_le_bytes([buffer[32], buffer[33], buffer[34], buffer[35]]),
            clusters_per_group: u32::from_le_bytes([buffer[36], buffer[37], buffer[38], buffer[39]]),
            inodes_per_group: u32::from_le_bytes([buffer[40], buffer[41], buffer[42], buffer[43]]),
            mtime: u32::from_le_bytes([buffer[44], buffer[45], buffer[46], buffer[47]]),
            wtime: u32::from_le_bytes([buffer[48], buffer[49], buffer[50], buffer[51]]),
            mnt_count: u16::from_le_bytes([buffer[52], buffer[53]]),
            max_mnt_count: u16::from_le_bytes([buffer[54], buffer[55]]),
            magic,
            state: u16::from_le_bytes([buffer[58], buffer[59]]),
            errors: u16::from_le_bytes([buffer[60], buffer[61]]),
            minor_rev_level: u16::from_le_bytes([buffer[62], buffer[63]]),
        })
    }
}

#[cfg(not(feature = "verus"))]
pub struct Ext4 {
    pub block_device: Option<crate::virtio_blk::VirtioBlkDriver>,
    pub mounted: bool,
    pub superblock: Option<Ext4Superblock>,
}

#[cfg(not(feature = "verus"))]
impl Ext4 {
    pub fn new() -> Self {
        Ext4 {
            block_device: None,
            mounted: false,
            superblock: None,
        }
    }

    pub fn new_with_device(dev: crate::virtio_blk::VirtioBlkDriver) -> Self {
        Ext4 {
            block_device: Some(dev),
            mounted: false,
            superblock: None,
        }
    }

    pub fn mount(&mut self) -> Result<(), ()> {
        if self.block_device.is_none() {
            return Err(());
        }

        if self.fsck().is_ok() {
            self.mounted = true;
            return Ok(());
        }

        Err(())
    }

    pub fn fsck(&mut self) -> Result<(), ()> {
        if self.block_device.is_none() {
            return Err(());
        }

        // Ext4 superblock is at offset 1024 bytes. If block size is 512, it's block 2.
        // We need at least 1024 bytes of the superblock structure.
        let mut superblock_buffer = [0u8; 1024];

        // This is a simplified read which assumes we get the buffer populated correctly
        // (the mock implementation of read_block will just return Ok)
        if self.read_block(2, &mut superblock_buffer).is_ok() {
            if let Some(sb) = Ext4Superblock::parse(&superblock_buffer) {
                self.superblock = Some(sb);
                return Ok(());
            }
        }

        Err(())
    }

    pub fn journal(&self) -> Result<(), ()> {
        Ok(())
    }

    pub fn read_block(&mut self, block: u64, _buffer: &mut [u8]) -> Result<(), ()> {
        if let Some(dev) = &mut self.block_device {
            if dev.read_sector(block, 0) {
                // In a real implementation, we would wait for the DMA transfer to complete
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
    fn test_ext4_superblock_parse() {
        let mut mock = [0u8; 1024];
        mock[56] = 0x53; // 0xEF53 magic
        mock[57] = 0xEF;
        mock[0] = 0x00;  // 1024 inodes
        mock[1] = 0x04;

        let sb = Ext4Superblock::parse(&mock).unwrap();
        assert_eq!(sb.magic, 0xEF53);
        assert_eq!(sb.inodes_count, 1024);
    }

    #[test]
    fn test_ext4_mount_failure() {
        let mut fs = Ext4::new();
        assert!(fs.mount().is_err());
        assert!(fs.fsck().is_err());
        assert!(fs.journal().is_ok());
    }

    #[test]
    fn test_ext4_fsck_failure_with_device() {
        let mut mmio_mock = [0u64; 1024];
        let base_addr = mmio_mock.as_mut_ptr() as usize;
        let drv = crate::virtio_blk::VirtioBlkDriver::new(4, 100, base_addr, (0, 2, 0));
        let mut fs_dev = Ext4::new_with_device(drv);

        // read_block returns Ok, but buffer is zeroes, so parsing magic 0xEF53 fails
        assert!(fs_dev.mount().is_err());
        assert!(fs_dev.fsck().is_err());
    }

    #[test]
    fn test_ext4_read_write() {
        let mut mmio_mock = [0u64; 1024];
        let base_addr = mmio_mock.as_mut_ptr() as usize;
        let drv = crate::virtio_blk::VirtioBlkDriver::new(4, 100, base_addr, (0, 2, 0));
        let mut fs_dev = Ext4::new_with_device(drv);

        let mut buf = [0u8; 512];
        assert!(fs_dev.read_block(0, &mut buf).is_ok());
        assert!(fs_dev.write_block(0, &buf).is_ok());

        // Out of bounds
        assert!(fs_dev.read_block(200, &mut buf).is_err());
        assert!(fs_dev.write_block(200, &buf).is_err());
    }
}
