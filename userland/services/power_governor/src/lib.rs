#![no_std]

extern crate alloc;

use inference_runtime::{InferenceEngine, Model, Tensor};

pub struct PowerGovernor {
    pub engine: InferenceEngine,
    pub model: Option<Model>,
    pub context: Option<usize>,
    pub cpu_freq: i32,
    pub screen_brightness: i32,
    pub peripheral_state: i32,
}

impl Default for PowerGovernor {
    fn default() -> Self {
        Self::new()
    }
}

impl PowerGovernor {
    pub fn new() -> Self {
        Self {
            engine: InferenceEngine::new(),
            model: None,
            context: None,
            cpu_freq: 2000,
            screen_brightness: 100,
            peripheral_state: 1, // 1 = On, 0 = Off/Suspend
        }
    }

    pub fn init(&mut self) -> Result<(), &'static str> {
        let model = self
            .engine
            .load_model_by_name("power_governor")
            .map_err(|_| "Failed to load model")?;
        let ctx = self
            .engine
            .init_execution_context(&model)
            .map_err(|_| "Failed to init execution context")?;
        self.model = Some(model);
        self.context = Some(ctx);
        Ok(())
    }

    pub fn fine_tune(&mut self, path: &str, weights: &[u8]) -> Result<(), &'static str> {
        let ctx = self.context.ok_or("Governor not initialized")?;
        self.engine
            .save_weights(ctx, path, weights)
            .map_err(|_| "Failed to save weights")?;
        Ok(())
    }

    pub fn analyze_and_adjust(
        &mut self,
        cpu_usage: u8,
        battery_level: u8,
        time_of_day: u8,
    ) -> Result<(), &'static str> {
        let ctx = self.context.ok_or("Governor not initialized")?;

        let input_data = alloc::vec![cpu_usage, battery_level, time_of_day];
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

        if bytes_written > 0 {
            // Read output tensor logic to adjust parameters based on AI inference
            // Mocking the tensor format. The model is assumed to output 3 bytes:
            // [desired_cpu_freq_scale, desired_screen_brightness, desired_peripheral_state]
            let freq_scale = out[0];
            let brightness = out[1];
            let peripheral = out[2];

            if freq_scale > 0 {
                self.cpu_freq = (freq_scale as i32) * 100;
            } else {
                self.cpu_freq = 2000; // default fallback
            }

            if brightness > 0 {
                self.screen_brightness = brightness as i32;
            } else {
                self.screen_brightness = 100; // default fallback
            }

            self.peripheral_state = if peripheral > 0 { 1 } else { 0 };

            #[cfg(target_arch = "wasm32")]
            {
                unsafe {
                    set_cpu_freq(self.cpu_freq);
                    set_screen_brightness(self.screen_brightness);
                    set_peripheral_power_state(1, self.peripheral_state);
                }
            }
        }

        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "wasi_snapshot_preview1")]
extern "C" {
    fn set_cpu_freq(freq_mhz: i32) -> i32;
    fn set_screen_brightness(level: i32) -> i32;
    fn set_peripheral_power_state(peripheral_id: i32, state: i32) -> i32;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_governor_init() {
        let mut governor = PowerGovernor::new();
        assert!(governor.init().is_ok());
        assert!(governor.model.is_some());
        assert!(governor.context.is_some());
    }

    #[test]
    fn test_power_governor_adjust() {
        let mut governor = PowerGovernor::new();
        governor.init().unwrap();

        governor.analyze_and_adjust(90, 20, 22).unwrap();

        // With the mock model bytes (b"mock_output"),
        // out[0] == b'm' == 109, out[1] == b'o' == 111, out[2] == b'c' == 99.
        // So cpu_freq should be 109 * 100 and screen_brightness 111.
        assert_eq!(governor.cpu_freq, 109 * 100);
        assert_eq!(governor.screen_brightness, 111);
        assert_eq!(governor.peripheral_state, 1);
    }

    #[test]
    fn test_power_governor_fine_tune() {
        let mut governor = PowerGovernor::new();
        governor.init().unwrap();
        assert!(governor.fine_tune("weights.bin", b"test_weights").is_ok());
    }
}
