extern crate alloc;
use alloc::string::{String, ToString};

pub struct NlDebugger {
    memory_snapshot: alloc::vec::Vec<u8>,
    variables: alloc::collections::BTreeMap<String, u64>,
}

impl NlDebugger {
    pub fn new() -> Self {
        Self {
            memory_snapshot: alloc::vec::Vec::new(),
            variables: alloc::collections::BTreeMap::new(),
        }
    }

    pub fn snapshot_memory(&mut self, mem: &[u8]) {
        self.memory_snapshot = mem.to_vec();
    }

    pub fn register_variable(&mut self, name: &str, addr: u64) {
        self.variables.insert(name.to_string(), addr);
    }

    // Mock natural language query resolution
    pub fn ask(&self, query: &str) -> String {
        // Very basic NLP mock
        if query.contains("value of") {
            for (name, addr) in &self.variables {
                if query.contains(name) {
                    if (*addr as usize) < self.memory_snapshot.len() {
                        let val = self.memory_snapshot[*addr as usize];
                        return alloc::format!("The value of {} is {}", name, val);
                    } else {
                        return alloc::format!("Variable {} is out of bounds", name);
                    }
                }
            }
            return "I couldn't find that variable.".to_string();
        }

        "I'm not sure how to answer that yet. Try asking 'what is the value of <variable>'."
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nl_debugger() {
        let mut debugger = NlDebugger::new();

        debugger.snapshot_memory(&[10, 20, 30, 42]);
        debugger.register_variable("counter", 3);

        let answer = debugger.ask("what is the value of counter?");
        assert_eq!(answer, "The value of counter is 42");

        let answer_unknown = debugger.ask("what is the value of speed?");
        assert_eq!(answer_unknown, "I couldn't find that variable.");
    }
}
