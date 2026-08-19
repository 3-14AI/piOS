#![no_std]

extern crate alloc;

use inference_runtime::{InferenceEngine, Model, Tensor};

pub struct IdsIps {
    pub engine: InferenceEngine,
    pub model: Option<Model>,
    pub context: Option<usize>,
    pub is_active: bool,
    pub threats_blocked: usize,
}

impl Default for IdsIps {
    fn default() -> Self {
        Self::new()
    }
}

impl IdsIps {
    pub fn new() -> Self {
        Self {
            engine: InferenceEngine::new(),
            model: None,
            context: None,
            is_active: false,
            threats_blocked: 0,
        }
    }

    pub fn init(&mut self) -> Result<(), &'static str> {
        let model = self
            .engine
            .load_model_by_name("ids_ips_model")
            .map_err(|_| "Failed to load model")?;
        let ctx = self
            .engine
            .init_execution_context(&model)
            .map_err(|_| "Failed to init execution context")?;
        self.model = Some(model);
        self.context = Some(ctx);
        self.is_active = true;
        Ok(())
    }

    pub fn analyze_and_block(
        &mut self,
        syscall_id: u8,
        arg1: u8,
        arg2: u8,
    ) -> Result<bool, &'static str> {
        if !self.is_active {
            return Err("IDS/IPS not active");
        }

        let ctx = self.context.ok_or("IDS/IPS not initialized")?;

        let input_data = alloc::vec![syscall_id, arg1, arg2];
        let tensor = Tensor::new(input_data, alloc::vec![3]);

        self.engine
            .set_input(ctx, 0, &tensor)
            .map_err(|_| "Failed to set input")?;
        self.engine.compute(ctx).map_err(|_| "Compute failed")?;

        let mut out = [0u8; 32];
        let bytes_written = self
            .engine
            .get_output(ctx, 0, &mut out)
            .map_err(|_| "Failed to get output")?;

        let mut blocked = false;
        if bytes_written > 0 {
            // Mock threshold
            let threat_score = out[0];
            if threat_score > 100 {
                self.threats_blocked += 1;
                blocked = true;
            }
        }

        Ok(blocked)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ids_ips_init() {
        let mut ids = IdsIps::new();
        assert!(ids.init().is_ok());
        assert!(ids.model.is_some());
        assert!(ids.context.is_some());
        assert!(ids.is_active);
    }

    #[test]
    fn test_ids_ips_analyze() {
        let mut ids = IdsIps::new();
        ids.init().unwrap();

        // mock outputs b"mock_output"
        // 'm' is 109, > 100, so it will be blocked
        let blocked = ids.analyze_and_block(1, 2, 3).unwrap();
        assert!(blocked);
        assert_eq!(ids.threats_blocked, 1);
    }
}
