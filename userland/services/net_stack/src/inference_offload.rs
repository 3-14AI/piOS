#![no_std]
extern crate alloc;

use crate::WasmNetStack;
use alloc::vec::Vec;

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
            stack.send_udp_broadcast(handle, port, tensor_payload);
            // Mock receiving the result back
            let mut result = Vec::new();
            result.extend_from_slice(b"offload_success");
            return Some(result);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inference_offload() {
        let mut stack = WasmNetStack::new();
        let mut offloader = InferenceOffloader::new();

        offloader.connect(&mut stack, 8080);
        assert!(offloader.connected);

        let result = offloader.offload_compute(&mut stack, &[0, 1, 2], 8080);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), b"offload_success");
    }
}
