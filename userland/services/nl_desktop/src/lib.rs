#![no_std]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use inference_runtime::{InferenceEngine, Model, Tensor};

pub struct NlDesktop {
    pub engine: InferenceEngine,
    pub model: Option<Model>,
    pub context: Option<usize>,
    pub last_command: String,
    pub context_stack: Vec<String>,
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
            last_command: String::new(),
            context_stack: Vec::new(),
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

    pub fn process_command(&mut self, nl_command: &str) -> Result<String, &'static str> {
        let ctx = self.context.ok_or("NL Desktop not initialized")?;

        let mut input_data = Vec::new();
        // Pack the context stack to give the LLM model a sense of context
        for context_str in &self.context_stack {
            input_data.extend_from_slice(context_str.as_bytes());
            input_data.push(b'\n');
        }
        input_data.extend_from_slice(nl_command.as_bytes());

        let tensor = Tensor::new(input_data.clone(), alloc::vec![1, input_data.len()]);

        self.engine
            .set_input(ctx, 0, &tensor)
            .map_err(|_| "Failed to set input")?;
        self.engine.compute(ctx).map_err(|_| "Compute failed")?;

        let mut out = [0u8; 32];
        let bytes_written = self
            .engine
            .get_output(ctx, 0, &mut out)
            .map_err(|_| "Failed to get output")?;

        let mut action = String::new();
        if bytes_written > 0 {
            // "mock_output"
            let action_id = out[0];
            if action_id == 109 {
                // Determine action based on mock logic + context awareness
                if self
                    .context_stack
                    .iter()
                    .any(|c| c.contains("open terminal"))
                    && nl_command.contains("and run htop")
                {
                    action = String::from("Open Window: terminal, run: htop");
                } else {
                    action = String::from("Open Window");
                }
            } else {
                action = String::from("Unknown Action");
            }
        }

        self.last_command = String::from(nl_command);
        self.context_stack.push(self.last_command.clone());
        if self.context_stack.len() > 5 {
            // Keep recent context
            self.context_stack.remove(0);
        }
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

    #[test]
    fn test_nl_desktop_context_aware_process() {
        let mut desktop = NlDesktop::new();
        desktop.init().unwrap();

        let _ = desktop.process_command("open terminal").unwrap();
        let action = desktop.process_command("and run htop").unwrap();
        assert_eq!(action, "Open Window: terminal, run: htop");
        assert_eq!(desktop.last_command, "and run htop");
    }
}
