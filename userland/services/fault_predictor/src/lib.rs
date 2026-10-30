extern crate alloc;
use alloc::string::String;
use inference_runtime::{InferenceEngine, Tensor};

pub struct FaultPredictor {
    pub engine: InferenceEngine,
    pub active_migrations: u32,
}

impl Default for FaultPredictor {
    fn default() -> Self {
        Self::new()
    }
}

impl FaultPredictor {
    pub fn new() -> Self {
        let engine = InferenceEngine::new();
        Self {
            engine,
            active_migrations: 0,
        }
    }

    pub fn predict_fault(&mut self, hw_telemetry: &str) -> Option<String> {
        let model = self.engine.load_model_by_name("fault_predictor").ok()?;
        let ctx = self.engine.init_execution_context(&model).ok()?;

        let tensor = Tensor::new(
            hw_telemetry.as_bytes().to_vec(),
            alloc::vec![hw_telemetry.len()],
        );

        if self.engine.set_input(ctx, 0, &tensor).is_ok() && self.engine.compute(ctx).is_ok() {
            let mut out_buffer = alloc::vec![0u8; 128];
            if let Ok(bytes_written) = self.engine.get_output(ctx, 0, &mut out_buffer) {
                let output = alloc::string::String::from_utf8_lossy(&out_buffer[..bytes_written])
                    .into_owned();

                if hw_telemetry.contains("ECC errors")
                    || hw_telemetry.contains("thermal throttling")
                {
                    self.active_migrations += 1;
                    return Some(String::from(
                        "migrating WASM contexts due to hardware fault prediction",
                    ));
                }

                return Some(output);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fault_prediction() {
        let mut predictor = FaultPredictor::new();
        assert_eq!(predictor.active_migrations, 0);

        let action = predictor.predict_fault("High frequency of ECC errors detected in RAM.");

        assert!(action.is_some());
        assert!(action.unwrap().contains("migrating"));
        assert_eq!(predictor.active_migrations, 1);
    }
}
