#![no_std]

extern crate alloc;

use inference_runtime::{InferenceEngine, Model, Tensor};

pub struct AppPreloader {
    pub engine: InferenceEngine,
    pub model: Option<Model>,
    pub context: Option<usize>,
    pub active_preload_slots: usize,
}

impl Default for AppPreloader {
    fn default() -> Self {
        Self::new()
    }
}

impl AppPreloader {
    pub fn new() -> Self {
        Self {
            engine: InferenceEngine::new(),
            model: None,
            context: None,
            active_preload_slots: 0,
        }
    }

    pub fn init(&mut self) -> Result<(), &'static str> {
        let model = self
            .engine
            .load_model_by_name("app_preloader_model")
            .map_err(|_| "Failed to load model")?;
        let ctx = self
            .engine
            .init_execution_context(&model)
            .map_err(|_| "Failed to init execution context")?;
        self.model = Some(model);
        self.context = Some(ctx);
        Ok(())
    }

    pub fn predict_and_preload(
        &mut self,
        time_of_day_hours: u8,
        last_app_id: u8,
    ) -> Result<u8, &'static str> {
        let ctx = self.context.ok_or("App preloader not initialized")?;

        let input_data = alloc::vec![time_of_day_hours, last_app_id];
        let tensor = Tensor::new(input_data, alloc::vec![2]);

        self.engine
            .set_input(ctx, 0, &tensor)
            .map_err(|_| "Failed to set input")?;
        self.engine.compute(ctx).map_err(|_| "Compute failed")?;

        let mut out = [0u8; 32];
        let bytes_written = self
            .engine
            .get_output(ctx, 0, &mut out)
            .map_err(|_| "Failed to get output")?;

        let mut predicted_app_id = 0;
        if bytes_written > 0 {
            predicted_app_id = out[0];
            self.active_preload_slots += 1;
        }

        Ok(predicted_app_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_preloader_init() {
        let mut preloader = AppPreloader::new();
        assert!(preloader.init().is_ok());
        assert!(preloader.model.is_some());
        assert!(preloader.context.is_some());
    }

    #[test]
    fn test_app_preloader_predict() {
        let mut preloader = AppPreloader::new();
        preloader.init().unwrap();

        // With mock "mock_output" output, out[0] is b'm' which is 109
        let app_id = preloader.predict_and_preload(10, 5).unwrap();
        assert_eq!(app_id, 109);
        assert_eq!(preloader.active_preload_slots, 1);
    }
}
