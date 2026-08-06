#![no_std]
#![allow(unused)]

extern crate alloc;

use alloc::vec::Vec;

pub struct AnomaliesMonitor {
    anomaly_threshold: usize,
    current_anomalies: usize,
}

impl Default for AnomaliesMonitor {
    fn default() -> Self {
        Self::new(5)
    }
}

impl AnomaliesMonitor {
    pub fn new(threshold: usize) -> Self {
        Self {
            anomaly_threshold: threshold,
            current_anomalies: 0,
        }
    }

    pub fn record_traffic_event(&mut self, is_anomalous: bool) {
        if is_anomalous {
            self.current_anomalies += 1;
        }
    }

    pub fn should_trigger_patch(&self) -> bool {
        self.current_anomalies >= self.anomaly_threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anomalies_monitor() {
        let mut monitor = AnomaliesMonitor::new(2);
        assert!(!monitor.should_trigger_patch());
        monitor.record_traffic_event(true);
        assert!(!monitor.should_trigger_patch());
        monitor.record_traffic_event(true);
        assert!(monitor.should_trigger_patch());
    }
}
