#[cfg(feature = "verus")]
use vstd::prelude::*;

#[cfg(feature = "verus")]
verus! {
    pub struct PerformanceMetrics {
        pub cpu_cycles: u64,
        pub memory_allocations: u64,
        pub ipc_messages_sent: u64,
        pub page_faults: u64,
    }

    impl PerformanceMetrics {
        pub fn new() -> (res: Self)
            ensures res.cpu_cycles == 0 && res.memory_allocations == 0 && res.ipc_messages_sent == 0 && res.page_faults == 0
        {
            Self {
                cpu_cycles: 0,
                memory_allocations: 0,
                ipc_messages_sent: 0,
                page_faults: 0,
            }
        }

        pub fn record_allocation(&mut self)
            requires old(self).memory_allocations < 0xffff_ffff_ffff_ffff
            ensures self.memory_allocations == old(self).memory_allocations + 1
        {
            self.memory_allocations = self.memory_allocations + 1;
        }

        pub fn record_ipc_message(&mut self)
            requires old(self).ipc_messages_sent < 0xffff_ffff_ffff_ffff
            ensures self.ipc_messages_sent == old(self).ipc_messages_sent + 1
        {
            self.ipc_messages_sent = self.ipc_messages_sent + 1;
        }

        pub fn record_page_fault(&mut self)
            requires old(self).page_faults < 0xffff_ffff_ffff_ffff
            ensures self.page_faults == old(self).page_faults + 1
        {
            self.page_faults = self.page_faults + 1;
        }
    }

    pub struct TelemetrySystem {
        pub metrics: PerformanceMetrics,
    }

    impl TelemetrySystem {
        pub fn new() -> (res: Self)
            ensures res.metrics.cpu_cycles == 0 && res.metrics.memory_allocations == 0 && res.metrics.ipc_messages_sent == 0 && res.metrics.page_faults == 0
        {
            Self {
                metrics: PerformanceMetrics::new(),
            }
        }

        pub fn get_metrics(&self) -> (res: PerformanceMetrics)
            ensures res.cpu_cycles == self.metrics.cpu_cycles &&
                    res.memory_allocations == self.metrics.memory_allocations &&
                    res.ipc_messages_sent == self.metrics.ipc_messages_sent &&
                    res.page_faults == self.metrics.page_faults
        {
            PerformanceMetrics {
                cpu_cycles: self.metrics.cpu_cycles,
                memory_allocations: self.metrics.memory_allocations,
                ipc_messages_sent: self.metrics.ipc_messages_sent,
                page_faults: self.metrics.page_faults,
            }
        }
    }
}

#[cfg(not(feature = "verus"))]
#[derive(Clone, Copy, Default, Debug)]
pub struct PerformanceMetrics {
    pub cpu_cycles: u64,
    pub memory_allocations: u64,
    pub ipc_messages_sent: u64,
    pub page_faults: u64,
}

#[cfg(not(feature = "verus"))]
impl Default for TelemetrySystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "verus"))]
impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            cpu_cycles: 0,
            memory_allocations: 0,
            ipc_messages_sent: 0,
            page_faults: 0,
        }
    }

    pub fn record_allocation(&mut self) {
        self.memory_allocations = self.memory_allocations.saturating_add(1);
    }

    pub fn record_ipc_message(&mut self) {
        self.ipc_messages_sent = self.ipc_messages_sent.saturating_add(1);
    }

    pub fn record_page_fault(&mut self) {
        self.page_faults = self.page_faults.saturating_add(1);
    }
}

#[cfg(not(feature = "verus"))]
pub struct TelemetrySystem {
    pub metrics: PerformanceMetrics,
}

#[cfg(not(feature = "verus"))]
impl TelemetrySystem {
    pub fn new() -> Self {
        Self {
            metrics: PerformanceMetrics::new(),
        }
    }

    pub fn get_metrics(&self) -> PerformanceMetrics {
        self.metrics
    }
}

#[cfg(test)]
#[cfg(not(feature = "verus"))]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_system() {
        let mut telemetry = TelemetrySystem::new();
        assert_eq!(telemetry.get_metrics().memory_allocations, 0);

        telemetry.metrics.record_allocation();
        assert_eq!(telemetry.get_metrics().memory_allocations, 1);

        telemetry.metrics.record_ipc_message();
        assert_eq!(telemetry.get_metrics().ipc_messages_sent, 1);

        telemetry.metrics.record_page_fault();
        assert_eq!(telemetry.get_metrics().page_faults, 1);
    }
}
