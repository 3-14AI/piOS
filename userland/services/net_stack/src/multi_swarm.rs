#![no_std]

use alloc::string::String;
use alloc::vec::Vec;

pub struct MultiSwarmProtocol {
    pub connected_clusters: Vec<String>,
}

impl Default for MultiSwarmProtocol {
    fn default() -> Self {
        Self::new()
    }
}

impl MultiSwarmProtocol {
    pub fn new() -> Self {
        Self {
            connected_clusters: Vec::new(),
        }
    }

    pub fn connect_cluster(&mut self, cluster_id: &str) -> bool {
        self.connected_clusters
            .push(alloc::format!("Cluster: {}", cluster_id));
        true
    }

    pub fn federate_models(&self) -> bool {
        !self.connected_clusters.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_swarm() {
        let mut protocol = MultiSwarmProtocol::new();
        assert_eq!(protocol.federate_models(), false);
        protocol.connect_cluster("swarm-alpha");
        assert_eq!(protocol.federate_models(), true);
    }
}
