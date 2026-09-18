#![no_std]

extern crate alloc;

use inference_runtime::{InferenceEngine, Model, Tensor};
use vector_db::{VectorDb, VectorRecord};

pub struct ProactiveAgent {
    pub engine: InferenceEngine,
    pub model: Option<Model>,
    pub context: Option<usize>,
    pub db: VectorDb,
    pub scheduler_quantum: i32,
    pub task_id: u8,
}

impl Default for ProactiveAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl ProactiveAgent {
    pub fn new() -> Self {
        Self {
            engine: InferenceEngine::new(),
            model: None,
            context: None,
            db: VectorDb::new(),
            scheduler_quantum: 15, // Non-blocking baseline
            task_id: 0,
        }
    }

    pub fn init(&mut self) -> Result<(), &'static str> {
        let model = self
            .engine
            .load_model_by_name("proactive_agent_model")
            .map_err(|_| "Failed to load model")?;
        let ctx = self
            .engine
            .init_execution_context(&model)
            .map_err(|_| "Failed to init execution context")?;
        self.model = Some(model);
        self.context = Some(ctx);
        Ok(())
    }

    pub fn analyze_and_act(&mut self, system_load: u8, user_activity: u8) -> Result<(), &'static str> {
        let ctx = self.context.ok_or("Agent not initialized")?;

        let input_data = alloc::vec![system_load, user_activity];
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

        if bytes_written > 0 {
            let recommended_quantum = out[0];
            let predicted_task = out[1];

            if recommended_quantum > 0 {
                self.scheduler_quantum = recommended_quantum as i32;
            }

            self.task_id = predicted_task;

            #[cfg(target_arch = "wasm32")]
            {
                unsafe {
                    set_scheduler_quantum(self.scheduler_quantum);
                }
            }

            // Share context via semantic namespace
            self.share_context(predicted_task);
        }

        Ok(())
    }

    fn share_context(&mut self, task_id: u8) {
        let task_desc = match task_id {
            1 => "system_cleanup",
            2 => "log_rotation",
            _ => "idle",
        };

        // We use string representation for vector DB ID.
        // A simple embedding generator mock to fulfill the context sharing requirement.
        let embedding = alloc::vec![task_id as f32, 1.0, 0.5];

        let record = VectorRecord {
            id: alloc::string::String::from("/semantic/context_share/task"),
            vector: embedding,
            metadata: Some(alloc::string::String::from(task_desc)),
        };
        let _ = self.db.insert(record);
    }
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "wasi_snapshot_preview1")]
extern "C" {
    fn set_scheduler_quantum(quantum: i32) -> i32;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proactive_agent_init() {
        let mut agent = ProactiveAgent::new();
        assert!(agent.init().is_ok());
        assert!(agent.model.is_some());
        assert!(agent.context.is_some());
    }

    #[test]
    fn test_proactive_agent_act() {
        let mut agent = ProactiveAgent::new();
        agent.init().unwrap();

        // Pass dummy values
        agent.analyze_and_act(50, 10).unwrap();

        // Mock inference engine always returns bytes b"mock_output"
        // out[0] == 'm' == 109, out[1] == 'o' == 111
        assert_eq!(agent.scheduler_quantum, 109);
        assert_eq!(agent.task_id, 111);

        // Check context share write
        assert_eq!(agent.db.len(), 1);
    }
}
