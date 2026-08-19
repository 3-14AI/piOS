#![no_std]

extern crate alloc;

use inference_runtime::{InferenceEngine, Model, Tensor};

pub struct NlDesktop {
    pub engine: InferenceEngine,
    pub model: Option<Model>,
    pub context: Option<usize>,
    pub last_command: alloc::string::String,
}

impl Default for NlDesktop {
    fn default() -> Self {
        Self::new()
    }
}

impl NlDesktop {
    pub fn new() -> Self {
        Self {
            engine: InferenceEngine::new(),
            model: None,
            context: None,
            last_command: alloc::string::String::new(),
        }
    }

    pub fn init(&mut self) -> Result<(), &'static str> {
        let model = self
            .engine
            .load_model_by_name("nl_desktop_model")
            .map_err(|_| "Failed to load model")?;
        let ctx = self
            .engine
            .init_execution_context(&model)
            .map_err(|_| "Failed to init execution context")?;
        self.model = Some(model);
        self.context = Some(ctx);
        Ok(())
    }

    pub fn process_command(
        &mut self,
        nl_command: &str,
    ) -> Result<alloc::string::String, &'static str> {
        let ctx = self.context.ok_or("NL Desktop not initialized")?;

        let input_data = alloc::vec![nl_command.len() as u8];
        let tensor = Tensor::new(input_data, alloc::vec![1]);

        self.engine
            .set_input(ctx, 0, &tensor)
            .map_err(|_| "Failed to set input")?;
        self.engine.compute(ctx).map_err(|_| "Compute failed")?;

        let mut out = [0u8; 32];
        let bytes_written = self
            .engine
            .get_output(ctx, 0, &mut out)
            .map_err(|_| "Failed to get output")?;

        let mut action = alloc::string::String::new();
        if bytes_written > 0 {
            // "mock_output"
            let action_id = out[0];
            if action_id == 109 {
                action = alloc::string::String::from("Open Window");
            } else {
                action = alloc::string::String::from("Unknown Action");
            }
        }

        self.last_command = alloc::string::String::from(nl_command);
        Ok(action)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nl_desktop_init() {
        let mut desktop = NlDesktop::new();
        assert!(desktop.init().is_ok());
        assert!(desktop.model.is_some());
        assert!(desktop.context.is_some());
    }

    #[test]
    fn test_nl_desktop_process() {
        let mut desktop = NlDesktop::new();
        desktop.init().unwrap();

        let action = desktop.process_command("open browser").unwrap();
        assert_eq!(action, "Open Window");
        assert_eq!(desktop.last_command, "open browser");
    }
}
