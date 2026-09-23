extern crate alloc;
use crate::WasmNetStack;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

pub struct TrustScore {
    pub node_id: u32,
    pub score: i32,
}

pub struct ZeroTrustManager {
    scores: BTreeMap<u32, i32>,
    threshold: i32,
}

impl ZeroTrustManager {
    pub fn new(threshold: i32) -> Self {
        Self {
            scores: BTreeMap::new(),
            threshold,
        }
    }

    pub fn update_score(&mut self, node_id: u32, delta: i32) {
        let entry = self.scores.entry(node_id).or_insert(100); // Default score is 100
        *entry += delta;
    }

    pub fn is_trusted(&self, node_id: u32) -> bool {
        if let Some(score) = self.scores.get(&node_id) {
            *score >= self.threshold
        } else {
            // New nodes are trusted by default if default score 100 >= threshold
            100 >= self.threshold
        }
    }

    pub fn restrict_capabilities(&self, node_id: u32) -> Vec<&'static str> {
        let mut caps = Vec::new();
        if self.is_trusted(node_id) {
            caps.push("network_access");
            caps.push("fs_access");
            caps.push("compute_access");
        } else {
            // Untrusted nodes get restricted caps
            caps.push("network_access_restricted");
        }
        caps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_trust_manager() {
        let mut mgr = ZeroTrustManager::new(50);

        assert!(mgr.is_trusted(1)); // default trusted
        assert_eq!(mgr.restrict_capabilities(1).len(), 3);

        mgr.update_score(2, -60); // score drops to 40
        assert!(!mgr.is_trusted(2));
        assert_eq!(mgr.restrict_capabilities(2), alloc::vec!["network_access_restricted"]);

        mgr.update_score(2, 20); // score up to 60
        assert!(mgr.is_trusted(2));
    }
}
