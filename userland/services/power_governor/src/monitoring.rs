#![allow(unused_imports)]
#![allow(dead_code)]

extern crate alloc;
use alloc::vec::Vec;

pub struct SystemMetrics {
    pub cpu_utilization: u8,
    pub memory_pressure: u8,
    pub network_traffic_kbps: u32,
}

pub struct MetricsMonitor {
    pub current_metrics: SystemMetrics,
}

impl Default for MetricsMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsMonitor {
    pub fn new() -> Self {
        Self {
            current_metrics: SystemMetrics {
                cpu_utilization: 0,
                memory_pressure: 0,
                network_traffic_kbps: 0,
            },
        }
    }

    pub fn poll_metrics(&mut self) -> Result<&SystemMetrics, &'static str> {
        #[cfg(target_arch = "wasm32")]
        unsafe {
            // Simulated WASI host calls to fetch real metrics
            self.current_metrics.cpu_utilization = sys_get_cpu_utilization() as u8;
            self.current_metrics.memory_pressure = sys_get_memory_pressure() as u8;
            self.current_metrics.network_traffic_kbps = sys_get_network_traffic() as u32;
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            // Fallback mock metrics for non-WASM execution / tests
            self.current_metrics.cpu_utilization = 45;
            self.current_metrics.memory_pressure = 60;
            self.current_metrics.network_traffic_kbps = 1500;
        }

        Ok(&self.current_metrics)
    }
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "wasi_snapshot_preview1")]
extern "C" {
    fn sys_get_cpu_utilization() -> i32;
    fn sys_get_memory_pressure() -> i32;
    fn sys_get_network_traffic() -> i32;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_polling() {
        let mut monitor = MetricsMonitor::new();
        let metrics = monitor.poll_metrics().unwrap();
        assert_eq!(metrics.cpu_utilization, 45);
        assert_eq!(metrics.memory_pressure, 60);
        assert_eq!(metrics.network_traffic_kbps, 1500);
    }
}
