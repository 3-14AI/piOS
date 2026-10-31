#![no_std]
extern crate alloc;

use crate::WasmNetStack;
use alloc::vec::Vec;
use inference_runtime::{InferenceEngine, Tensor};

pub struct InferenceOffloader {
    pub connected: bool,
    pub socket_handle: Option<smoltcp::iface::SocketHandle>,
}

impl Default for InferenceOffloader {
    fn default() -> Self {
        Self::new()
    }
}

impl InferenceOffloader {
    pub fn new() -> Self {
        Self {
            connected: false,
            socket_handle: None,
        }
    }

    pub fn connect(&mut self, stack: &mut WasmNetStack, port: u16) {
        self.socket_handle = Some(stack.add_udp_broadcast_socket(port));
        self.connected = true;
    }

    pub fn offload_compute(
        &self,
        stack: &mut WasmNetStack,
        tensor_payload: &[u8],
        port: u16,
    ) -> Option<Vec<u8>> {
        if !self.connected {
            return None;
        }

        if let Some(handle) = self.socket_handle {
            // Build the packet: OFFLOAD:data
            let mut packet = Vec::new();
            packet.extend_from_slice(b"OFFLOAD:");
            packet.extend_from_slice(tensor_payload);

            stack.send_udp_broadcast(handle, port, &packet);

            // Mock receiving the computed result back
            let mut result = Vec::new();
            result.extend_from_slice(b"RESULT:computed_tensor_data");
            return Some(result);
        }

        None
    }

    pub fn handle_incoming_request(&self, data: &[u8]) -> Option<Vec<u8>> {
        if data.starts_with(b"OFFLOAD:") {
            let tensor_data = &data[8..];
            // In a real scenario, this node would invoke `inference_runtime`
            // on its local GPU/NPU and return the processed result.
            // For now, we mock the local computation step.

            let mut result = Vec::new();
            result.extend_from_slice(b"RESULT:computed_tensor_data");
            return Some(result);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inference_offload_request() {
        let mut stack = WasmNetStack::new();
        let mut offloader = InferenceOffloader::new();

        offloader.connect(&mut stack, 8080);
        assert!(offloader.connected);

        let result = offloader.offload_compute(&mut stack, &[0, 1, 2, 3], 8080);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), b"RESULT:computed_tensor_data");
    }

    #[test]
    fn test_inference_offload_handle_incoming() {
        let offloader = InferenceOffloader::new();
        let mut request = alloc::vec::Vec::new();
        request.extend_from_slice(b"OFFLOAD:");
        request.extend_from_slice(&[0, 1, 2, 3]);

        let response = offloader.handle_incoming_request(&request);
        assert!(response.is_some());
        assert_eq!(response.unwrap(), b"RESULT:computed_tensor_data");
    }
}
