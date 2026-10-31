#![no_std]
extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use inference_runtime::{InferenceEngine, Tensor};

pub struct GoalWorkflowManager {
    pub engine: InferenceEngine,
    pub active_goals: Vec<String>,
}

impl Default for GoalWorkflowManager {
    fn default() -> Self {
        Self::new()
    }
}

impl GoalWorkflowManager {
    pub fn new() -> Self {
        let engine = InferenceEngine::new();
        Self {
            engine,
            active_goals: Vec::new(),
        }
    }

    pub fn submit_intent(&mut self, intent: &str) {
        self.active_goals.push(alloc::format!("Intent: {}", intent));
    }

    pub fn process_workflows(&mut self) -> Option<Vec<String>> {
        if self.active_goals.is_empty() {
            return None;
        }

        let mut actions = Vec::new();
        let model = self.engine.load_model_by_name("goal_workflow_planner").ok()?;
        let ctx = self.engine.init_execution_context(&model).ok()?;

        for goal in &self.active_goals {
            let tensor = Tensor::new(goal.as_bytes().to_vec(), alloc::vec![goal.len()]);
            if self.engine.set_input(ctx, 0, &tensor).is_ok() && self.engine.compute(ctx).is_ok() {
                let mut out_buffer = alloc::vec![0u8; 1024];
                if let Ok(_bytes_written) = self.engine.get_output(ctx, 0, &mut out_buffer) {
                    actions.push(alloc::format!("Action planned for: {}", goal));
                }
            }
        }

        self.active_goals.clear();
        Some(actions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_processing() {
        let mut manager = GoalWorkflowManager::new();
        manager.submit_intent("Compile code and move to VFS");

        let actions = manager.process_workflows().unwrap();
        assert_eq!(actions.len(), 1);
        assert!(actions[0].contains("Compile code"));
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
