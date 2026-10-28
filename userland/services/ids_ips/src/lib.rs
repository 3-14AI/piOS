#![no_std]

extern crate alloc;

use inference_runtime::{InferenceEngine, Model, Tensor};

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "wasi_snapshot_preview1")]
extern "C" {
    fn sys_deploy_firewall_rule(rule_ptr: *const u8, rule_len: i32) -> i32;
}

#[cfg(not(target_arch = "wasm32"))]
unsafe fn sys_deploy_firewall_rule(_rule_ptr: *const u8, _rule_len: i32) -> i32 {
    0
}
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

    pub fn analyze_packet_and_generate_rules(
        &mut self,
        packet_data: &[u8],
    ) -> Result<bool, &'static str> {
        if !self.is_active {
            return Err("IDS/IPS not active");
        }

        let ctx = self.context.ok_or("IDS/IPS not initialized")?;

        let input_data = packet_data.to_vec();
        let len = input_data.len();
        let tensor = Tensor::new(input_data, alloc::vec![len]);

        self.engine
            .set_input(ctx, 0, &tensor)
            .map_err(|_| "Failed to set input")?;
        self.engine.compute(ctx).map_err(|_| "Compute failed")?;

        let mut out = [0u8; 128];
        let bytes_written = self
            .engine
            .get_output(ctx, 0, &mut out)
            .map_err(|_| "Failed to get output")?;

        let mut deployed = false;
        if bytes_written > 0 {
            let anomaly_score = out[0];
            if anomaly_score > 80 {
                // Generate a dummy rule block from output
                let rule_len = core::cmp::min(bytes_written - 1, 127);
                if rule_len > 0 {
                    let rule = &out[1..(1 + rule_len)];

                    unsafe {
                        sys_deploy_firewall_rule(rule.as_ptr(), rule.len() as i32);
                    }
                }

                self.threats_blocked += 1;
                deployed = true;
            }
        }

        Ok(deployed)
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
    fn test_ids_ips_analyze_packet() {
        let mut ids = IdsIps::new();
        ids.init().unwrap();

        let packet = [
            0x45, 0x00, 0x00, 0x28, 0x00, 0x01, 0x00, 0x00, 0x40, 0x06, 0x7c, 0xcd, 0x7f, 0x00,
            0x00, 0x01, 0x7f, 0x00, 0x00, 0x01,
        ];

        // mock outputs b"mock_output"
        // 'm' is 109, which is > 80, so it will deploy the firewall rule
        let deployed = ids.analyze_packet_and_generate_rules(&packet).unwrap();
        assert!(deployed);
        assert_eq!(ids.threats_blocked, 1);
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

