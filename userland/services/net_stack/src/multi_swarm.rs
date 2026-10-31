#![no_std]
extern crate alloc;

use crate::WasmNetStack;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::time::Duration;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum MultiSwarmState {
    Isolated,
    Federated,
}

pub struct MultiSwarmNode {
    pub cluster_id: String,
    pub ip: [u8; 4],
    pub known_clusters: BTreeMap<String, [u8; 4]>,
    pub socket_handle: Option<smoltcp::iface::SocketHandle>,
    pub state: MultiSwarmState,
}

impl MultiSwarmNode {
    pub fn new(cluster_id: &str, ip: [u8; 4]) -> Self {
        Self {
            cluster_id: cluster_id.to_string(),
            ip,
            known_clusters: BTreeMap::new(),
            socket_handle: None,
            state: MultiSwarmState::Isolated,
        }
    }

    pub fn bind(&mut self, stack: &mut WasmNetStack, port: u16) {
        self.socket_handle = Some(stack.add_udp_broadcast_socket(port));
    }

    pub fn broadcast_discovery(&self, stack: &mut WasmNetStack, port: u16) {
        if let Some(handle) = self.socket_handle {
            let msg = alloc::format!("DISCOVER:{}", self.cluster_id);
            stack.send_udp_broadcast(handle, port, msg.as_bytes());
        }
    }

    pub fn receive_packet(&mut self, data: &[u8], sender_ip: [u8; 4]) {
        if let Ok(msg) = core::str::from_utf8(data) {
            if msg.starts_with("DISCOVER:") {
                let id = msg.trim_start_matches("DISCOVER:");
                if id != self.cluster_id {
                    self.known_clusters.insert(id.to_string(), sender_ip);
                    if !self.known_clusters.is_empty() {
                        // Need at least 1 other to federate
                        self.state = MultiSwarmState::Federated;
                    }
                }
            }
        }
    }

    pub fn get_state(&self) -> MultiSwarmState {
        self.state.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_swarm_discovery() {
        let mut stack = WasmNetStack::new();
        let mut node = MultiSwarmNode::new("alpha-swarm", [192, 168, 1, 10]);
        node.bind(&mut stack, 8081);

        assert_eq!(node.get_state(), MultiSwarmState::Isolated);

        node.broadcast_discovery(&mut stack, 8081);

        node.receive_packet(b"DISCOVER:beta-swarm", [192, 168, 1, 20]);
        assert_eq!(node.get_state(), MultiSwarmState::Federated);
        assert!(node.known_clusters.contains_key("beta-swarm"));
    }
}
