#![allow(dead_code)]

#[cfg(feature = "verus")]
use vstd::prelude::*;

#[cfg(feature = "verus")]
verus! {
    pub struct AmdIntelGpuDriver {
        pub vram_size: u64,
    }

    impl AmdIntelGpuDriver {
        pub fn new(vram_size: u64) -> (d: Self)
            ensures d.vram_size == vram_size
        {
            AmdIntelGpuDriver { vram_size }
        }
    }
}

#[cfg(not(feature = "verus"))]
pub struct AmdIntelGpuDriver {
    pub vram_size: u64,
}

#[cfg(not(feature = "verus"))]
impl AmdIntelGpuDriver {
    pub fn new(vram_size: u64) -> Self {
        AmdIntelGpuDriver { vram_size }
    }
}

#[cfg(not(feature = "verus"))]
extern crate alloc;

#[cfg(not(feature = "verus"))]
pub struct KmsConnector {
    pub id: u32,
    pub connected: bool,
}

#[cfg(not(feature = "verus"))]
pub struct KmsCrtc {
    pub id: u32,
    pub width: u32,
    pub height: u32,
    pub fb_id: Option<u32>,
}

#[cfg(not(feature = "verus"))]
pub trait DrmDevice {
    fn get_connectors(&self) -> alloc::vec::Vec<KmsConnector>;
    fn get_crtcs(&self) -> alloc::vec::Vec<KmsCrtc>;
    fn set_crtc(
        &mut self,
        crtc_id: u32,
        fb_id: u32,
        connector_ids: &[u32],
    ) -> Result<(), &'static str>;
}

#[cfg(not(feature = "verus"))]
impl DrmDevice for AmdIntelGpuDriver {
    fn get_connectors(&self) -> alloc::vec::Vec<KmsConnector> {
        alloc::vec![KmsConnector {
            id: 1,
            connected: true
        }]
    }

    fn get_crtcs(&self) -> alloc::vec::Vec<KmsCrtc> {
        alloc::vec![KmsCrtc {
            id: 1,
            width: 1920,
            height: 1080,
            fb_id: None
        }]
    }

    fn set_crtc(
        &mut self,
        _crtc_id: u32,
        _fb_id: u32,
        _connector_ids: &[u32],
    ) -> Result<(), &'static str> {
        Ok(())
    }
}

#[cfg(feature = "verus")]
verus! {
    pub struct KmsConnector {
        pub id: u32,
        pub connected: bool,
    }

    pub struct KmsCrtc {
        pub id: u32,
        pub width: u32,
        pub height: u32,
        pub fb_id: Option<u32>,
    }
}

#[cfg(not(feature = "verus"))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_driver() {
        let drv = AmdIntelGpuDriver::new(4096);
        assert_eq!(drv.vram_size, 4096);
    }

    #[test]
    fn test_drm_kms() {
        let mut drv = AmdIntelGpuDriver::new(8192);
        let connectors = drv.get_connectors();
        assert_eq!(connectors.len(), 1);
        assert!(connectors[0].connected);

        let crtcs = drv.get_crtcs();
        assert_eq!(crtcs.len(), 1);
        assert_eq!(crtcs[0].width, 1920);

        let res = drv.set_crtc(1, 100, &[1]);
        assert!(res.is_ok());
    }
}
