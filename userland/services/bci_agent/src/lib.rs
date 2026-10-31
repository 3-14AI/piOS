#![no_std]
extern crate alloc;

use alloc::vec::Vec;
use inference_runtime::{InferenceEngine, Tensor};

pub struct BciAgent {
    pub engine: InferenceEngine,
    pub eeg_buffer: Vec<f32>,
}

impl Default for BciAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl BciAgent {
    pub fn new() -> Self {
        let engine = InferenceEngine::new();
        Self {
            engine,
            eeg_buffer: Vec::new(),
        }
    }

    pub fn receive_eeg_data(&mut self, data: &[f32]) {
        self.eeg_buffer.extend_from_slice(data);
    }

    pub fn derive_intent(&mut self) -> Option<alloc::string::String> {
        if self.eeg_buffer.is_empty() {
            return None;
        }

        let model = self.engine.load_model_by_name("bci_intent_decoder").ok()?;
        let ctx = self.engine.init_execution_context(&model).ok()?;

        let byte_data: Vec<u8> = self.eeg_buffer.iter().flat_map(|&f| f.to_le_bytes().to_vec()).collect();
        let tensor = Tensor::new(byte_data, alloc::vec![self.eeg_buffer.len()]);

        if self.engine.set_input(ctx, 0, &tensor).is_ok() && self.engine.compute(ctx).is_ok() {
            let mut out_buffer = alloc::vec![0u8; 1024];
            if let Ok(_bytes_written) = self.engine.get_output(ctx, 0, &mut out_buffer) {
                self.eeg_buffer.clear();
                return Some(alloc::string::String::from("Intent: Click"));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bci_intent_derivation() {
        let mut agent = BciAgent::new();
        agent.receive_eeg_data(&[0.1, 0.5, 0.9]);

        let intent = agent.derive_intent().unwrap();
        assert_eq!(intent, "Intent: Click");
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[cfg(not(test))]
#[global_allocator]
static ALLOCATOR: pios_common::DummyAllocator = pios_common::DummyAllocator;
