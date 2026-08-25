slint::include_modules!();
use nl_desktop::NlDesktop;

pub struct GenerativeUI {
    app: AppWindow,
    desktop_ai: NlDesktop,
}

impl GenerativeUI {
    pub fn new() -> Result<Self, slint::PlatformError> {
        let app = AppWindow::new()?;
        let mut desktop_ai = NlDesktop::new();
        let _ = desktop_ai.init();
        Ok(Self { app, desktop_ai })
    }

    pub fn set_text(&self, text: &str) {
        self.app.set_generative_text(text.into());
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
}
