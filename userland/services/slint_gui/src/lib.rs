slint::include_modules!();

pub struct GenerativeUI {
    app: AppWindow,
}

impl GenerativeUI {
    pub fn new() -> Result<Self, slint::PlatformError> {
        let app = AppWindow::new()?;
        Ok(Self { app })
    }

    pub fn set_text(&self, text: &str) {
        self.app.set_generative_text(text.into());
    }

    pub fn run(&self) -> Result<(), slint::PlatformError> {
        self.app.run()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    // Slint needs a minimal platform in no_std/test environments if standard event loop isn't used.
    // For unit testing properties, we can often just instantiate it if the default backend falls back to testing/software.
    // Setting up a dummy platform to satisfy slint in test mode:

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
}
