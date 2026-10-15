#![allow(clippy::empty_loop)]

extern crate alloc;

slint::include_modules!();

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "wasi_snapshot_preview1")]
extern "C" {
    fn sys_get_framebuffer(fb_info_ptr: *mut u8) -> i32;
    fn sys_flush_framebuffer(buf_ptr: *const u8, buf_len: i32) -> i32;
    fn sys_poll_input_event(event_ptr: *mut u8) -> i32;
}

#[cfg(not(target_arch = "wasm32"))]
unsafe fn sys_get_framebuffer(_fb_info_ptr: *mut u8) -> i32 {
    0 // WASI_ERRNO_SUCCESS
}

#[cfg(not(target_arch = "wasm32"))]
unsafe fn sys_flush_framebuffer(_buf_ptr: *const u8, _buf_len: i32) -> i32 {
    0 // WASI_ERRNO_SUCCESS
}

#[cfg(not(target_arch = "wasm32"))]
unsafe fn sys_poll_input_event(_event_ptr: *mut u8) -> i32 {
    6 // WASI_ERRNO_AGAIN
}

use alloc::rc::Rc;
use slint::platform::software_renderer::{MinimalSoftwareWindow, RepaintBufferType};
use slint::Rgb8Pixel;

struct HardwarePlatform {
    window: Rc<MinimalSoftwareWindow>,
}

impl slint::platform::Platform for HardwarePlatform {
    fn create_window_adapter(
        &self,
    ) -> Result<Rc<dyn slint::platform::WindowAdapter>, slint::PlatformError> {
        Ok(self.window.clone())
    }

    fn duration_since_start(&self) -> core::time::Duration {
        core::time::Duration::from_millis(0)
    }

    fn run_event_loop(&self) -> Result<(), slint::PlatformError> {
        Ok(())
    }
}

fn init_baremetal_platform() -> Option<(Rc<MinimalSoftwareWindow>, slint::Timer)> {
    let mut fb_info = [0u8; 24];
    let res = unsafe { sys_get_framebuffer(fb_info.as_mut_ptr()) };

    if res == 0 {
        let mut width_buf = [0u8; 4];
        let mut height_buf = [0u8; 4];
        let mut stride_buf = [0u8; 4];
        width_buf.copy_from_slice(&fb_info[0..4]);
        height_buf.copy_from_slice(&fb_info[4..8]);
        stride_buf.copy_from_slice(&fb_info[8..12]);
        let width = u32::from_le_bytes(width_buf) as usize;
        let height = u32::from_le_bytes(height_buf) as usize;
        let _stride = u32::from_le_bytes(stride_buf) as usize;

        let window = MinimalSoftwareWindow::new(RepaintBufferType::ReusedBuffer);
        window.set_size(slint::PhysicalSize::new(width as u32, height as u32));

        let platform = HardwarePlatform {
            window: window.clone(),
        };

        if slint::platform::set_platform(alloc::boxed::Box::new(platform)).is_err() {
            return None;
        }

        let render_timer = slint::Timer::default();
        let win_clone = window.clone();
        render_timer.start(
            slint::TimerMode::Repeated,
            core::time::Duration::from_millis(30),
            move || {
                let mut needs_flush = false;
                win_clone.draw_if_needed(|renderer| {
                    let mut buffer = alloc::vec![Rgb8Pixel::default(); width * height];
                    renderer.render(&mut buffer, width);

                    let bytes = unsafe {
                        core::slice::from_raw_parts(
                            buffer.as_ptr() as *const u8,
                            buffer.len() * core::mem::size_of::<Rgb8Pixel>(),
                        )
                    };
                    let chunk_size = 4096;
                    for chunk in bytes.chunks(chunk_size) {
                        unsafe {
                            sys_flush_framebuffer(chunk.as_ptr(), chunk.len() as i32);
                        }
                    }
                    needs_flush = true;
                });

                let _ = needs_flush;
            },
        );

        return Some((window, render_timer));
    }
    None
}

pub struct TelemetryDashboard {
    app: DashboardWindow,
    #[cfg(not(test))]
    #[allow(dead_code)]
    _timer: Option<slint::Timer>,
}

impl TelemetryDashboard {
    pub fn new() -> Result<Self, slint::PlatformError> {
        let baremetal_info = init_baremetal_platform();

        let app = DashboardWindow::new()?;

        #[cfg(not(test))]
        let _timer = if let Some((_, t)) = baremetal_info {
            Some(t)
        } else {
            None
        };

        Ok(Self {
            app,
            #[cfg(not(test))]
            _timer,
        })
    }

    pub fn set_cpu_usage(&self, usage: &str) {
        self.app.set_cpu_usage(usage.into());
    }

    pub fn set_mem_usage(&self, usage: &str) {
        self.app.set_mem_usage(usage.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_dashboard_init() {
        let dashboard_result = TelemetryDashboard::new();
        assert!(dashboard_result.is_ok());
        let dashboard = dashboard_result.unwrap();

        dashboard.set_cpu_usage("15%");
        dashboard.set_mem_usage("256 MB");

        assert_eq!(dashboard.app.get_cpu_usage(), "15%");
        assert_eq!(dashboard.app.get_mem_usage(), "256 MB");
    }
}
