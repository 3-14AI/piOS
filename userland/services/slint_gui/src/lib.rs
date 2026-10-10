slint::include_modules!();
use nl_desktop::NlDesktop;

use inference_runtime::VisionModel;

pub struct GenerativeUI {
    app: AppWindow,
    desktop_ai: NlDesktop,
    vision_model: VisionModel,
    #[cfg(not(test))]
    #[allow(dead_code)]
    _timer: Option<slint::Timer>,
}

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

use slint::platform::software_renderer::{MinimalSoftwareWindow, RepaintBufferType};
use slint::Rgb8Pixel;
use std::rc::Rc;

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
        // We use Instant::now() which maps to clock_time_get(MONOTONIC) in WASI.
        // If it fails or is unavailable in our specific `#![no_std]` environment,
        // we fallback to a static tick counter to ensure Slint timers advance.
        #[cfg(target_arch = "wasm32")]
        {
            static START_TIME: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
            let start = START_TIME.get_or_init(|| std::time::Instant::now());
            start.elapsed()
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            core::time::Duration::default()
        }
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
        let stride = u32::from_le_bytes(stride_buf) as usize;

        let window = MinimalSoftwareWindow::new(RepaintBufferType::ReusedBuffer);
        window.set_size(slint::PhysicalSize::new(width as u32, height as u32));

        let platform = HardwarePlatform {
            window: window.clone(),
        };
        let _ = slint::platform::set_platform(Box::new(platform));

        let timer = slint::Timer::default();
        let window_clone = window.clone();

        // Allocate buffer once and map to 32bpp for the kernel driver
        let mut render_buf = vec![Rgb8Pixel::default(); stride * height];
        let mut mapped_buf = vec![0u8; stride * height * 4];

        timer.start(
            slint::TimerMode::Repeated,
            core::time::Duration::from_millis(16),
            move || {
                let mut event_buf = [0u8; 12];
                while unsafe { sys_poll_input_event(event_buf.as_mut_ptr()) } == 0 {
                    let ev_type = u32::from_le_bytes([
                        event_buf[0],
                        event_buf[1],
                        event_buf[2],
                        event_buf[3],
                    ]);
                    let code = u16::from_le_bytes([event_buf[4], event_buf[5]]);
                    let _value = i32::from_le_bytes([
                        event_buf[8],
                        event_buf[9],
                        event_buf[10],
                        event_buf[11],
                    ]);

                    if ev_type == 1 {
                        // Key
                        // Simulate mapping standard keycodes to Slint keys or logic
                        // For now, we can just dispatch a space key for any input if code > 0 to see interaction
                        if code > 0 {
                            let text = slint::SharedString::from(" ");
                            window_clone.dispatch_event(slint::platform::WindowEvent::KeyPressed {
                                text: text.clone(),
                            });
                            window_clone.dispatch_event(
                                slint::platform::WindowEvent::KeyReleased { text: text },
                            );
                        }
                    }
                }

                window_clone.draw_if_needed(|renderer| {
                    renderer.render(render_buf.as_mut_slice(), stride);

                    let mut i = 0;
                    for p in &render_buf {
                        mapped_buf[i] = p.b;
                        mapped_buf[i + 1] = p.g;
                        mapped_buf[i + 2] = p.r;
                        mapped_buf[i + 3] = 255; // Alpha
                        i += 4;
                    }

                    unsafe {
                        sys_flush_framebuffer(mapped_buf.as_ptr(), mapped_buf.len() as i32);
                    }
                });
            },
        );

        Some((window, timer))
    } else {
        None
    }
}

impl GenerativeUI {
    pub fn new() -> Result<Self, slint::PlatformError> {
        #[cfg(not(test))]
        let baremetal_info = init_baremetal_platform();

        let app = AppWindow::new()?;

        #[cfg(not(test))]
        let _timer = if let Some((_, t)) = baremetal_info {
            Some(t)
        } else {
            None
        };

        let mut desktop_ai = NlDesktop::new();
        let _ = desktop_ai.init();
        let vision_model = VisionModel::new(1, "gui_vision_model");
        Ok(Self {
            app,
            desktop_ai,
            vision_model,
            #[cfg(not(test))]
            _timer,
        })
    }

    pub fn set_text(&self, text: &str) {
        self.app.set_generative_text(text.into());
    }

    pub fn process_visual_input(&self, image_data: &[u8]) {
        if let Ok(description) = self.vision_model.process_image(image_data) {
            self.app.set_generative_text(description.into());
        }
    }

    pub fn handle_nl_command(&mut self, command: &str) {
        if command.contains("button") {
            self.app.set_active_element("button".into());
            self.app.set_button_text("AI Button".into());
            return;
        }

        if let Ok(action) = self.desktop_ai.process_command(command) {
            if action == "Open Window" || command.contains("window") {
                self.app.set_active_element("window".into());
                self.app.set_window_title("AI Generated Window".into());
            } else {
                self.app.set_active_element("text".into());
                self.app.set_generative_text(action.into());
            }
        } else {
            self.app.set_active_element("text".into());
            self.app
                .set_generative_text("Failed to process command".into());
        }
    }

    pub fn run(&self) -> Result<(), slint::PlatformError> {
        self.app.run()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::rc::Rc;

    struct TestPlatform {
        window: Rc<slint::platform::software_renderer::MinimalSoftwareWindow>,
    }

    impl slint::platform::Platform for TestPlatform {
        fn create_window_adapter(
            &self,
        ) -> Result<Rc<dyn slint::platform::WindowAdapter>, slint::PlatformError> {
            Ok(self.window.clone())
        }
        fn duration_since_start(&self) -> core::time::Duration {
            core::time::Duration::default()
        }
    }

    fn init_test_platform() {
        let window = slint::platform::software_renderer::MinimalSoftwareWindow::new(
            slint::platform::software_renderer::RepaintBufferType::NewBuffer,
        );
        let _ = slint::platform::set_platform(Box::new(TestPlatform { window }));
    }

    #[test]
    fn test_generative_ui_creation() {
        init_test_platform();
        let ui = GenerativeUI::new().unwrap();
        assert_eq!(ui.app.get_generative_text(), "Welcome to Generative UI");
    }

    #[test]
    fn test_generative_ui_set_text() {
        init_test_platform();
        let ui = GenerativeUI::new().unwrap();
        ui.set_text("Hello AI");
        assert_eq!(ui.app.get_generative_text(), "Hello AI");
    }

    #[test]
    fn test_generative_ui_nl_command_window() {
        init_test_platform();
        let mut ui = GenerativeUI::new().unwrap();
        ui.handle_nl_command("open browser");
        // Due to nl_desktop mock logic, "open browser" results in "Open Window" action
        assert_eq!(ui.app.get_active_element(), "window");
        assert_eq!(ui.app.get_window_title(), "AI Generated Window");
    }

    #[test]
    fn test_generative_ui_nl_command_button() {
        init_test_platform();
        let mut ui = GenerativeUI::new().unwrap();
        ui.handle_nl_command("create a button");
        assert_eq!(ui.app.get_active_element(), "button");
        assert_eq!(ui.app.get_button_text(), "AI Button");
    }

    #[test]
    fn test_generative_ui_process_visual_input() {
        init_test_platform();
        let ui = GenerativeUI::new().unwrap();
        ui.process_visual_input(b"dummy_image");
        assert_eq!(
            ui.app.get_generative_text(),
            "A simulated view of a user looking at the screen."
        );
    }
}
