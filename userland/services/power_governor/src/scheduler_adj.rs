#![allow(unused_imports)]
#![allow(dead_code)]

extern crate alloc;

pub struct SchedulerAdjuster {
    pub current_quantum_ms: u32,
}

impl Default for SchedulerAdjuster {
    fn default() -> Self {
        Self::new()
    }
}

impl SchedulerAdjuster {
    pub fn new() -> Self {
        Self {
            current_quantum_ms: 10,
        }
    }

    pub fn adjust_quantum(
        &mut self,
        cpu_utilization: u8,
        memory_pressure: u8,
    ) -> Result<u32, &'static str> {
        // Adjust scheduler quanta using heuristics
        if cpu_utilization > 80 || memory_pressure > 80 {
            self.current_quantum_ms = 5; // Smaller quantum for higher responsiveness under load
        } else if cpu_utilization < 20 {
            self.current_quantum_ms = 20; // Larger quantum for power saving when idle
        } else {
            self.current_quantum_ms = 10; // Default
        }

        #[cfg(target_arch = "wasm32")]
        unsafe {
            sys_set_scheduler_quantum(self.current_quantum_ms as i32);
        }

        Ok(self.current_quantum_ms)
    }
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "wasi_snapshot_preview1")]
extern "C" {
    fn sys_set_scheduler_quantum(quantum_ms: i32) -> i32;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adjust_quantum() {
        let mut adj = SchedulerAdjuster::new();
        assert_eq!(adj.adjust_quantum(90, 50).unwrap(), 5);
        assert_eq!(adj.adjust_quantum(10, 50).unwrap(), 20);
        assert_eq!(adj.adjust_quantum(50, 50).unwrap(), 10);
        assert_eq!(adj.adjust_quantum(50, 90).unwrap(), 5);
    }
}
