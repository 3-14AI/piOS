extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use crate::WasmNetStack;
use smoltcp::iface::SocketHandle;
use smoltcp::socket::tcp::{Socket as TcpSocket, SocketBuffer as TcpSocketBuffer, State as TcpState};
use smoltcp::wire::IpAddress;

pub struct DistributedComputeClient {
    socket_handle: SocketHandle,
}

impl DistributedComputeClient {
    pub fn new(stack: &mut WasmNetStack) -> Self {
        let rx_buffer = TcpSocketBuffer::new(alloc::vec![0; 4096]);
        let tx_buffer = TcpSocketBuffer::new(alloc::vec![0; 4096]);
        let socket = TcpSocket::new(rx_buffer, tx_buffer);
        let socket_handle = stack.sockets.add(socket);
        Self { socket_handle }
    }

    pub fn discover(&mut self, stack: &mut WasmNetStack, addr: IpAddress, port: u16) -> Result<(), &'static str> {
        let socket = stack.sockets.get_mut::<TcpSocket>(self.socket_handle);
        socket.connect(stack.interface.context(), (addr, port), port + 1)
            .map_err(|_| "Failed to discover peer")
    }

    pub fn send_workload(&mut self, stack: &mut WasmNetStack, workload: &[u8]) -> Result<(), &'static str> {
        let socket = stack.sockets.get_mut::<TcpSocket>(self.socket_handle);
        if socket.state() != TcpState::Established {
            return Err("Not connected to peer");
        }
        if socket.can_send() {
            socket.send_slice(workload).map_err(|_| "Failed to send workload")?;
            Ok(())
        } else {
            Err("Cannot send workload right now")
        }
    }

    pub fn receive_result(&mut self, stack: &mut WasmNetStack) -> Result<Vec<u8>, &'static str> {
        let socket = stack.sockets.get_mut::<TcpSocket>(self.socket_handle);
        if socket.can_recv() {
            let mut buf = alloc::vec![0; 4096];
            let len = socket.recv_slice(&mut buf).map_err(|_| "Failed to receive result")?;
            if len == 0 {
                return Err("No data received");
            }
            buf.truncate(len);
            Ok(buf)
        } else {
            Err("Cannot receive result right now")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use smoltcp::wire::IpAddress;

    #[test]
    fn test_distributed_compute() {
        let mut stack = WasmNetStack::new();
        let mut client = DistributedComputeClient::new(&mut stack);
        let addr = IpAddress::v4(192, 168, 1, 2);

        let _ = client.discover(&mut stack, addr, 8080);
        let _ = client.send_workload(&mut stack, b"mock_workload");
        let _ = client.receive_result(&mut stack);
    }
}
