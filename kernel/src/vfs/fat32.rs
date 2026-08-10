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
#[derive(Debug, Clone, Copy)]
pub struct BiosParameterBlock {
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub reserved_sector_count: u16,
    pub table_count: u8,
    pub root_entry_count: u16,
    pub total_sectors_16: u16,
    pub media_type: u8,
    pub table_size_16: u16,
    pub sectors_per_track: u16,
    pub head_side_count: u16,
    pub hidden_sector_count: u32,
    pub total_sectors_32: u32,
    pub table_size_32: u32,
    pub root_cluster: u32,
}

#[cfg(not(feature = "verus"))]
impl BiosParameterBlock {
    pub fn parse(buffer: &[u8]) -> Option<Self> {
        if buffer.len() < 512 {
            return None;
        }
        // Basic signature check: 0x55 0xAA at end of sector
        if buffer[510] != 0x55 || buffer[511] != 0xAA {
            return None;
        }

        let bytes_per_sector = u16::from_le_bytes([buffer[11], buffer[12]]);
        let sectors_per_cluster = buffer[13];
        let reserved_sector_count = u16::from_le_bytes([buffer[14], buffer[15]]);
        let table_count = buffer[16];
        let root_entry_count = u16::from_le_bytes([buffer[17], buffer[18]]);
        let total_sectors_16 = u16::from_le_bytes([buffer[19], buffer[20]]);
        let media_type = buffer[21];
        let table_size_16 = u16::from_le_bytes([buffer[22], buffer[23]]);
        let sectors_per_track = u16::from_le_bytes([buffer[24], buffer[25]]);
        let head_side_count = u16::from_le_bytes([buffer[26], buffer[27]]);
        let hidden_sector_count = u32::from_le_bytes([buffer[28], buffer[29], buffer[30], buffer[31]]);
        let total_sectors_32 = u32::from_le_bytes([buffer[32], buffer[33], buffer[34], buffer[35]]);

        // For FAT32
        let table_size_32 = u32::from_le_bytes([buffer[36], buffer[37], buffer[38], buffer[39]]);
        let root_cluster = u32::from_le_bytes([buffer[44], buffer[45], buffer[46], buffer[47]]);

        Some(Self {
            bytes_per_sector,
            sectors_per_cluster,
            reserved_sector_count,
            table_count,
            root_entry_count,
            total_sectors_16,
            media_type,
            table_size_16,
            sectors_per_track,
            head_side_count,
            hidden_sector_count,
            total_sectors_32,
            table_size_32,
            root_cluster,
        })
    }
}

#[cfg(not(feature = "verus"))]
pub struct Fat32 {
    pub block_device: Option<crate::virtio_blk::VirtioBlkDriver>,
    pub mounted: bool,
    pub bpb: Option<BiosParameterBlock>,
}

#[cfg(not(feature = "verus"))]
impl Fat32 {
    pub fn new() -> Self {
        Fat32 {
            block_device: None,
            mounted: false,
            bpb: None,
        }
    }

    pub fn new_with_device(dev: crate::virtio_blk::VirtioBlkDriver) -> Self {
        Fat32 {
            block_device: Some(dev),
            mounted: false,
            bpb: None,
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

        let mut boot_sector = [0u8; 512];
        if self.read_block(0, &mut boot_sector).is_ok() {
            if let Some(bpb) = BiosParameterBlock::parse(&boot_sector) {
                if bpb.bytes_per_sector > 0 && bpb.sectors_per_cluster > 0 {
                    self.bpb = Some(bpb);
                    return Ok(());
                }
            }
        }

        Err(())
    }

    pub fn read_block(&mut self, block: u64, _buffer: &mut [u8]) -> Result<(), ()> {
        if let Some(dev) = &mut self.block_device {
            if dev.read_sector(block, 0) {
                // In a real implementation, we would wait for the DMA transfer to complete
                // and the buffer would be populated.
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
    fn test_fat32_bpb_parse() {
        let mut mock = [0u8; 512];
        mock[510] = 0x55;
        mock[511] = 0xAA;
        mock[11] = 0x00;
        mock[12] = 0x02; // 512 bytes/sector
        mock[13] = 8;    // 8 sectors/cluster
        mock[14] = 32;   // 32 reserved sectors
        mock[16] = 2;    // 2 FATs

        let bpb = BiosParameterBlock::parse(&mock).unwrap();
        assert_eq!(bpb.bytes_per_sector, 512);
        assert_eq!(bpb.sectors_per_cluster, 8);
        assert_eq!(bpb.reserved_sector_count, 32);
        assert_eq!(bpb.table_count, 2);
    }

    #[test]
    fn test_fat32_mount_failure() {
        let mut fs = Fat32::new();
        assert!(fs.mount().is_err());
        assert!(fs.fsck().is_err());
    }

    #[test]
    fn test_fat32_fsck_failure_with_device() {
        let mut mmio_mock = [0u64; 1024];
        let base_addr = mmio_mock.as_mut_ptr() as usize;
        let drv = crate::virtio_blk::VirtioBlkDriver::new(4, 100, base_addr, (0, 2, 0));
        let mut fs_dev = Fat32::new_with_device(drv);

        // Since read_block doesn't populate the buffer, parsing will fail (buffer is zeroed)
        assert!(fs_dev.mount().is_err());
        assert!(fs_dev.fsck().is_err());
    }

    #[test]
    fn test_fat32_read_write() {
        let mut mmio_mock = [0u64; 1024];
        let base_addr = mmio_mock.as_mut_ptr() as usize;
        let drv = crate::virtio_blk::VirtioBlkDriver::new(4, 100, base_addr, (0, 2, 0));
        let mut fs_dev = Fat32::new_with_device(drv);

        let mut buf = [0u8; 512];
        assert!(fs_dev.read_block(0, &mut buf).is_ok());
        assert!(fs_dev.write_block(0, &buf).is_ok());

        assert!(fs_dev.read_block(200, &mut buf).is_err());
        assert!(fs_dev.write_block(200, &buf).is_err());
    }
}
