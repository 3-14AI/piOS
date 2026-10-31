#![no_std]
extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use inference_runtime::{InferenceEngine, Tensor};

pub struct KnowledgeGraph {
    pub engine: InferenceEngine,
    pub knowledge: Vec<String>,
}

impl Default for KnowledgeGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl KnowledgeGraph {
    pub fn new() -> Self {
        let engine = InferenceEngine::new();
        Self {
            engine,
            knowledge: Vec::new(),
        }
    }

    pub fn ingest_log(&mut self, log: &str) {
        self.knowledge.push(alloc::format!("Log: {}", log));
    }

    pub fn ingest_context(&mut self, context: &str) {
        self.knowledge.push(alloc::format!("Context: {}", context));
    }

    pub fn ingest_hardware_state(&mut self, state: &str) {
        self.knowledge
            .push(alloc::format!("HardwareState: {}", state));
    }

    pub fn build_graph(&mut self) -> Option<Vec<String>> {
        if self.knowledge.is_empty() {
            return None;
        }

        let mut graph = Vec::new();
        let model = self
            .engine
            .load_model_by_name("knowledge_graph_builder")
            .ok()?;
        let ctx = self.engine.init_execution_context(&model).ok()?;

        for item in &self.knowledge {
            let tensor = Tensor::new(item.as_bytes().to_vec(), alloc::vec![item.len()]);
            if self.engine.set_input(ctx, 0, &tensor).is_ok() && self.engine.compute(ctx).is_ok() {
                let mut out_buffer = alloc::vec![0u8; 1024];
                if let Ok(_bytes_written) = self.engine.get_output(ctx, 0, &mut out_buffer) {
                    graph.push(alloc::format!("Node: {}", item));
                }
            }
        }

        self.knowledge.clear();
        Some(graph)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knowledge_graph_ingestion() {
        let mut graph = KnowledgeGraph::new();
        graph.ingest_log("System booted");
        graph.ingest_hardware_state("CPU 50% usage");

        let nodes = graph.build_graph().unwrap();
        assert_eq!(nodes.len(), 2);
        assert!(nodes[0].contains("System booted"));
        assert!(nodes[1].contains("CPU 50% usage"));
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
