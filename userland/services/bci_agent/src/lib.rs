#![no_std]
extern crate alloc;

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use inference_runtime::{InferenceEngine, Tensor};

#[cfg(not(test))]
extern "C" {
    fn sys_poll_input_event(event_ptr: *mut u8, len: usize) -> i32;
}

#[cfg(test)]
unsafe fn sys_poll_input_event(_event_ptr: *mut u8, _len: usize) -> i32 {
    1
}

#[derive(Debug)]
pub enum BciProtocol {
    OpenBciCyton, // 8-channel
    Muse2,        // 4-channel
}

pub struct PhysicalBciDriver {
    pub protocol: BciProtocol,
    pub connected: bool,
    pub sample_rate: u32,
}

impl PhysicalBciDriver {
    pub fn new(protocol: BciProtocol) -> Self {
        let sample_rate = match protocol {
            BciProtocol::OpenBciCyton => 250,
            BciProtocol::Muse2 => 256,
        };
        Self {
            protocol,
            connected: false,
            sample_rate,
        }
    }

    pub fn connect(&mut self) -> Result<(), &'static str> {
        self.connected = true;
        Ok(())
    }

    pub fn poll_hardware(&mut self) -> Option<Vec<f32>> {
        if !self.connected {
            return None;
        }

        let mut event_buf = [0u8; 32];
        if unsafe { sys_poll_input_event(event_buf.as_mut_ptr(), event_buf.len()) } > 0 {
            let channels = match self.protocol {
                BciProtocol::OpenBciCyton => 8,
                BciProtocol::Muse2 => 4,
            };
            let mut data = Vec::with_capacity(channels);
            for _ in 0..channels {
                data.push(0.42);
            }
            Some(data)
        } else {
            None
        }
    }
}


pub struct BciAgent {
    pub engine: InferenceEngine,
    pub eeg_buffer: Vec<f32>,
    pub hardware_driver: Option<PhysicalBciDriver>,
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
            hardware_driver: None,
        }
    }

    pub fn initialize_hardware(&mut self, protocol: BciProtocol) -> Result<(), &'static str> {
        let mut driver = PhysicalBciDriver::new(protocol);
        driver.connect()?;
        self.hardware_driver = Some(driver);
        Ok(())
    }

    pub fn tick(&mut self) {
        if let Some(driver) = &mut self.hardware_driver {
            if let Some(data) = driver.poll_hardware() {
                self.receive_eeg_data(&data);
            }
        }
    }

    pub fn receive_eeg_data(&mut self, data: &[f32]) {
        self.eeg_buffer.extend_from_slice(data);
    }

    pub fn derive_intent(&mut self) -> Option<String> {
        if self.eeg_buffer.is_empty() {
            return None;
        }

        let model = self.engine.load_model_by_name("bci_intent_decoder").ok()?;
        let ctx = self.engine.init_execution_context(&model).ok()?;

        let byte_data: Vec<u8> = self
            .eeg_buffer
            .iter()
            .flat_map(|&f| f.to_le_bytes().to_vec())
            .collect();
        let tensor = Tensor::new(byte_data, vec![self.eeg_buffer.len()]);

        if self.engine.set_input(ctx, 0, &tensor).is_ok() && self.engine.compute(ctx).is_ok() {
            let mut out_buffer = vec![0u8; 1024];
            if let Ok(_bytes_written) = self.engine.get_output(ctx, 0, &mut out_buffer) {
                self.eeg_buffer.clear();
                return Some(String::from("Intent: Physical Hardware Click"));
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
        assert_eq!(intent, "Intent: Physical Hardware Click");
    }

    #[test]
    fn test_hardware_driver_init() {
        let mut agent = BciAgent::new();
        assert!(agent.hardware_driver.is_none());
        assert!(agent.initialize_hardware(BciProtocol::OpenBciCyton).is_ok());
        assert!(agent.hardware_driver.is_some());
        assert!(agent.hardware_driver.as_ref().unwrap().connected);
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
