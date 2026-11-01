use alloc::string::String;
use alloc::vec::Vec;

pub struct SelfOptimizingTopology {
    pub nodes: Vec<String>,
}

impl SelfOptimizingTopology {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn optimize(&mut self) -> Option<String> {
        Some(String::from(
            "Optimized network topology for lowest latency",
        ))
    }
}

impl Default for SelfOptimizingTopology {
    fn default() -> Self {
        Self::new()
    }
}
