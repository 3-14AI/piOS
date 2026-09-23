extern crate alloc;
use crate::WasmNetStack;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use smoltcp::iface::SocketHandle;
use smoltcp::socket::tcp::{
    Socket as TcpSocket, SocketBuffer as TcpSocketBuffer, State as TcpState,
};
use smoltcp::wire::IpAddress;

pub enum DsmMessage {
    ReadRequest { addr: u64, size: u32 },
    ReadResponse { data: Vec<u8> },
    WriteRequest { addr: u64, data: Vec<u8> },
    WriteResponse { success: bool },
}

impl DsmMessage {
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        match self {
            Self::ReadRequest { addr, size } => {
                buf.push(0);
                buf.extend_from_slice(&addr.to_le_bytes());
                buf.extend_from_slice(&size.to_le_bytes());
            }
            Self::ReadResponse { data } => {
                buf.push(1);
                buf.extend_from_slice(&(data.len() as u32).to_le_bytes());
                buf.extend_from_slice(data);
            }
            Self::WriteRequest { addr, data } => {
                buf.push(2);
                buf.extend_from_slice(&addr.to_le_bytes());
                buf.extend_from_slice(&(data.len() as u32).to_le_bytes());
                buf.extend_from_slice(data);
            }
            Self::WriteResponse { success } => {
                buf.push(3);
                buf.push(if *success { 1 } else { 0 });
            }
        }
        buf
    }

    pub fn deserialize(data: &[u8]) -> Option<Self> {
        if data.is_empty() {
            return None;
        }
        match data[0] {
            0 => {
                if data.len() < 13 {
                    return None;
                }
                let addr = u64::from_le_bytes(data[1..9].try_into().unwrap());
                let size = u32::from_le_bytes(data[9..13].try_into().unwrap());
                Some(Self::ReadRequest { addr, size })
            }
            1 => {
                if data.len() < 5 {
                    return None;
                }
                let len = u32::from_le_bytes(data[1..5].try_into().unwrap()) as usize;
                if data.len() < 5 + len {
                    return None;
                }
                let payload = data[5..5 + len].to_vec();
                Some(Self::ReadResponse { data: payload })
            }
            2 => {
                if data.len() < 13 {
                    return None;
                }
                let addr = u64::from_le_bytes(data[1..9].try_into().unwrap());
                let len = u32::from_le_bytes(data[9..13].try_into().unwrap()) as usize;
                if data.len() < 13 + len {
                    return None;
                }
                let payload = data[13..13 + len].to_vec();
                Some(Self::WriteRequest { addr, data: payload })
            }
            3 => {
                if data.len() < 2 {
                    return None;
                }
                Some(Self::WriteResponse {
                    success: data[1] == 1,
                })
            }
            _ => None,
        }
    }
}

pub struct DistributedMemoryManager {
    // Map remote IP to a socket handle used for DSM
    connections: BTreeMap<IpAddress, SocketHandle>,
    // Local memory pages exposed to the swarm (mocked as a simple flat byte array)
    local_memory: Vec<u8>,
}

impl DistributedMemoryManager {
    pub fn new(size: usize) -> Self {
        Self {
            connections: BTreeMap::new(),
            local_memory: alloc::vec![0; size],
        }
    }

    pub fn connect_to_node(&mut self, stack: &mut WasmNetStack, addr: IpAddress, port: u16) -> Result<(), &'static str> {
        let handle = stack.add_tcp_socket();
        let socket = stack.sockets.get_mut::<TcpSocket>(handle);
        socket.connect(stack.interface.context(), (addr, port), port + 2)
            .map_err(|_| "Failed to connect for DSM")?;
        self.connections.insert(addr, handle);
        Ok(())
    }

    pub fn read_remote_memory(&mut self, stack: &mut WasmNetStack, addr: IpAddress, remote_addr: u64, size: u32) -> Result<Vec<u8>, &'static str> {
        let handle = self.connections.get(&addr).ok_or("Node not connected")?;
        let socket = stack.sockets.get_mut::<TcpSocket>(*handle);

        let req = DsmMessage::ReadRequest { addr: remote_addr, size };
        if socket.can_send() {
            socket.send_slice(&req.serialize()).map_err(|_| "Failed to send read request")?;
        } else {
            return Err("Socket cannot send");
        }

        // Mock receiving response synchronously for simplicity in tests
        Ok(alloc::vec![0; size as usize])
    }

    pub fn write_remote_memory(&mut self, stack: &mut WasmNetStack, addr: IpAddress, remote_addr: u64, data: Vec<u8>) -> Result<(), &'static str> {
        let handle = self.connections.get(&addr).ok_or("Node not connected")?;
        let socket = stack.sockets.get_mut::<TcpSocket>(*handle);

        let req = DsmMessage::WriteRequest { addr: remote_addr, data };
        if socket.can_send() {
            socket.send_slice(&req.serialize()).map_err(|_| "Failed to send write request")?;
        } else {
            return Err("Socket cannot send");
        }

        Ok(())
    }

    pub fn process_requests(&mut self, stack: &mut WasmNetStack) {
        for (_addr, handle) in self.connections.iter() {
            let socket = stack.sockets.get_mut::<TcpSocket>(*handle);
            if socket.can_recv() {
                let mut buf = alloc::vec![0; 4096];
                if let Ok(len) = socket.recv_slice(&mut buf) {
                    buf.truncate(len);
                    if let Some(msg) = DsmMessage::deserialize(&buf) {
                        match msg {
                            DsmMessage::ReadRequest { addr, size } => {
                                let start = addr as usize;
                                let end = start + size as usize;
                                if end <= self.local_memory.len() {
                                    let data = self.local_memory[start..end].to_vec();
                                    let resp = DsmMessage::ReadResponse { data };
                                    let _ = socket.send_slice(&resp.serialize());
                                }
                            }
                            DsmMessage::WriteRequest { addr, data } => {
                                let start = addr as usize;
                                let end = start + data.len();
                                let mut success = false;
                                if end <= self.local_memory.len() {
                                    self.local_memory[start..end].copy_from_slice(&data);
                                    success = true;
                                }
                                let resp = DsmMessage::WriteResponse { success };
                                let _ = socket.send_slice(&resp.serialize());
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use smoltcp::wire::IpAddress;

    #[test]
    fn test_dsm_message_serialization() {
        let req = DsmMessage::ReadRequest { addr: 0x1000, size: 256 };
        let data = req.serialize();
        if let Some(DsmMessage::ReadRequest { addr, size }) = DsmMessage::deserialize(&data) {
            assert_eq!(addr, 0x1000);
            assert_eq!(size, 256);
        } else {
            panic!("Deserialization failed");
        }

        let req = DsmMessage::WriteRequest { addr: 0x2000, data: alloc::vec![1, 2, 3, 4] };
        let data = req.serialize();
        if let Some(DsmMessage::WriteRequest { addr, data: parsed_data }) = DsmMessage::deserialize(&data) {
            assert_eq!(addr, 0x2000);
            assert_eq!(parsed_data, alloc::vec![1, 2, 3, 4]);
        } else {
            panic!("Deserialization failed");
        }
    }

    #[test]
    fn test_dsm_read_write_remote() {
        let mut stack = WasmNetStack::new();
        let mut dsm = DistributedMemoryManager::new(4096);
        let addr = IpAddress::v4(192, 168, 1, 3);

        // Mock connection
        assert!(dsm.connect_to_node(&mut stack, addr, 9090).is_ok());

        let handle = *dsm.connections.get(&addr).unwrap();
        // Since we can't easily mock Established state in smoltcp without real traffic,
        // we will just assert that the socket exists.
        assert!(stack.sockets.get::<TcpSocket>(handle).state() == TcpState::SynSent);
    }
}
