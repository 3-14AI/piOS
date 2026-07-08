#![allow(dead_code)]

#[cfg(feature = "verus")]
use vstd::prelude::*;

#[cfg(feature = "verus")]
verus! {
    pub struct FramebufferDriver {
        pub width: u32,
        pub height: u32,
        pub stride: u32,
        pub base_address: u64,
        pub size: u64,
        pub bpp: u8,
    }

    impl FramebufferDriver {
        pub fn new(width: u32, height: u32, stride: u32, base_address: u64, size: u64, bpp: u8) -> (d: Self)
            ensures d.width == width && d.height == height && d.stride == stride && d.base_address == base_address && d.size == size && d.bpp == bpp
        {
            FramebufferDriver { width, height, stride, base_address, size, bpp }
        }

        // We use an unverified external stub or assume it performs the same function as the std version.
        // For Verus we can just declare the signature in verus block, or mark it as external_body.
        #[verifier(external_body)]
        pub fn write_pixel(&mut self, x: u32, y: u32, color: u32) {
            // Unverified stub for verifier
        }
    }

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

    pub struct IntelGpuDriver {
        pub vram_size: u64,
        pub initialized: bool,
    }

    impl IntelGpuDriver {
        pub fn new(vram_size: u64) -> (d: Self)
            ensures d.vram_size == vram_size && d.initialized == true
        {
            IntelGpuDriver { vram_size, initialized: true }
        }
    }
}

#[cfg(not(feature = "verus"))]
pub struct FramebufferDriver {
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub base_address: u64,
    pub size: u64,
    pub bpp: u8,
}

#[cfg(not(feature = "verus"))]
impl FramebufferDriver {
    pub fn new(
        width: u32,
        height: u32,
        stride: u32,
        base_address: u64,
        size: u64,
        bpp: u8,
    ) -> Self {
        FramebufferDriver {
            width,
            height,
            stride,
            base_address,
            size,
            bpp,
        }
    }

    pub fn write_pixel(&mut self, x: u32, y: u32, color: u32) {
        if x < self.width && y < self.height {
            let byte_per_pixel = (self.bpp / 8) as usize;
            let offset = (y * self.stride + x) as usize * byte_per_pixel;
            if (offset as u64) < self.size {
                let ptr = (self.base_address as usize + offset) as *mut u32;
                unsafe {
                    core::ptr::write_volatile(ptr, color);
                }
            }
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
pub struct IntelGpuDriver {
    pub vram_size: u64,
    pub initialized: bool,
}

#[cfg(not(feature = "verus"))]
impl IntelGpuDriver {
    pub fn new(vram_size: u64) -> Self {
        IntelGpuDriver {
            vram_size,
            initialized: true,
        }
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

#[cfg(not(feature = "verus"))]
impl DrmDevice for IntelGpuDriver {
    fn get_connectors(&self) -> alloc::vec::Vec<KmsConnector> {
        alloc::vec![KmsConnector {
            id: 2,
            connected: true
        }]
    }

    fn get_crtcs(&self) -> alloc::vec::Vec<KmsCrtc> {
        alloc::vec![KmsCrtc {
            id: 2,
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
    fn test_framebuffer_driver() {
        let fb = FramebufferDriver::new(1024, 768, 1024, 0x10000000, 1024 * 768 * 4, 32);
        assert_eq!(fb.width, 1024);
        assert_eq!(fb.height, 768);
        assert_eq!(fb.stride, 1024);
        assert_eq!(fb.base_address, 0x10000000);
        assert_eq!(fb.size, 1024 * 768 * 4);
        assert_eq!(fb.bpp, 32);

        // We cannot test write_pixel easily since it writes to physical memory addresses,
        // which will page fault in userspace test runner. So we just ensure it compiles.
    }

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

    #[test]
    fn test_intel_gpu_driver() {
        let mut drv = IntelGpuDriver::new(4096);
        assert_eq!(drv.vram_size, 4096);
        assert!(drv.initialized);

        let connectors = drv.get_connectors();
        assert_eq!(connectors.len(), 1);
        assert!(connectors[0].connected);

        let crtcs = drv.get_crtcs();
        assert_eq!(crtcs.len(), 1);
        assert_eq!(crtcs[0].width, 1920);

        let res = drv.set_crtc(2, 200, &[2]);
        assert!(res.is_ok());
    }
}
