extern crate alloc;
use crate::WasmNetStack;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use smoltcp::iface::SocketHandle;
use smoltcp::socket::tcp::{
    Socket as TcpSocket, SocketBuffer as TcpSocketBuffer, State as TcpState,
};
use smoltcp::wire::IpAddress;

pub enum IdeMessage {
    FileUpdate { path: String, content: String },
    CursorMove { user_id: u32, line: u32, col: u32 },
}

impl IdeMessage {
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        match self {
            Self::FileUpdate { path, content } => {
                buf.push(0);
                let path_bytes = path.as_bytes();
                buf.extend_from_slice(&(path_bytes.len() as u32).to_le_bytes());
                buf.extend_from_slice(path_bytes);
                let content_bytes = content.as_bytes();
                buf.extend_from_slice(&(content_bytes.len() as u32).to_le_bytes());
                buf.extend_from_slice(content_bytes);
            }
            Self::CursorMove { user_id, line, col } => {
                buf.push(1);
                buf.extend_from_slice(&user_id.to_le_bytes());
                buf.extend_from_slice(&line.to_le_bytes());
                buf.extend_from_slice(&col.to_le_bytes());
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
                if data.len() < 5 {
                    return None;
                }
                let path_len = u32::from_le_bytes(data[1..5].try_into().unwrap()) as usize;
                if data.len() < 5 + path_len + 4 {
                    return None;
                }
                let path = String::from_utf8(data[5..5 + path_len].to_vec()).ok()?;

                let content_start = 5 + path_len;
                let content_len =
                    u32::from_le_bytes(data[content_start..content_start + 4].try_into().unwrap())
                        as usize;
                if data.len() < content_start + 4 + content_len {
                    return None;
                }
                let content = String::from_utf8(
                    data[content_start + 4..content_start + 4 + content_len].to_vec(),
                )
                .ok()?;

                Some(Self::FileUpdate { path, content })
            }
            1 => {
                if data.len() < 13 {
                    return None;
                }
                let user_id = u32::from_le_bytes(data[1..5].try_into().unwrap());
                let line = u32::from_le_bytes(data[5..9].try_into().unwrap());
                let col = u32::from_le_bytes(data[9..13].try_into().unwrap());
                Some(Self::CursorMove { user_id, line, col })
            }
            _ => None,
        }
    }
}

pub struct CollaborativeIdeClient {
    connections: BTreeMap<IpAddress, SocketHandle>,
    pub local_files: BTreeMap<String, String>,
    pub cursors: BTreeMap<u32, (u32, u32)>, // user_id -> (line, col)
}

impl Default for CollaborativeIdeClient {
    fn default() -> Self {
        Self::new()
    }
}

impl CollaborativeIdeClient {
    pub fn new() -> Self {
        Self {
            connections: BTreeMap::new(),
            local_files: BTreeMap::new(),
            cursors: BTreeMap::new(),
        }
    }

    pub fn connect_to_ide(
        &mut self,
        stack: &mut WasmNetStack,
        addr: IpAddress,
        port: u16,
    ) -> Result<(), &'static str> {
        let handle = stack.add_tcp_socket();
        let socket = stack.sockets.get_mut::<TcpSocket>(handle);
        socket
            .connect(stack.interface.context(), (addr, port), port + 3)
            .map_err(|_| "Failed to connect for IDE")?;
        self.connections.insert(addr, handle);
        Ok(())
    }

    pub fn send_file_update(
        &mut self,
        stack: &mut WasmNetStack,
        addr: IpAddress,
        path: String,
        content: String,
    ) -> Result<(), &'static str> {
        let handle = self.connections.get(&addr).ok_or("Node not connected")?;
        let socket = stack.sockets.get_mut::<TcpSocket>(*handle);

        let req = IdeMessage::FileUpdate { path, content };
        if socket.can_send() {
            socket
                .send_slice(&req.serialize())
                .map_err(|_| "Failed to send IDE update")?;
        } else {
            return Err("Socket cannot send");
        }
        Ok(())
    }

    pub fn send_cursor_move(
        &mut self,
        stack: &mut WasmNetStack,
        addr: IpAddress,
        user_id: u32,
        line: u32,
        col: u32,
    ) -> Result<(), &'static str> {
        let handle = self.connections.get(&addr).ok_or("Node not connected")?;
        let socket = stack.sockets.get_mut::<TcpSocket>(*handle);

        let req = IdeMessage::CursorMove { user_id, line, col };
        if socket.can_send() {
            socket
                .send_slice(&req.serialize())
                .map_err(|_| "Failed to send cursor move")?;
        } else {
            return Err("Socket cannot send");
        }
        Ok(())
    }

    pub fn process_updates(&mut self, stack: &mut WasmNetStack) {
        for (_addr, handle) in self.connections.iter() {
            let socket = stack.sockets.get_mut::<TcpSocket>(*handle);
            if socket.can_recv() {
                let mut buf = alloc::vec![0; 4096];
                if let Ok(len) = socket.recv_slice(&mut buf) {
                    buf.truncate(len);
                    if let Some(msg) = IdeMessage::deserialize(&buf) {
                        match msg {
                            IdeMessage::FileUpdate { path, content } => {
                                self.local_files.insert(path, content);
                            }
                            IdeMessage::CursorMove { user_id, line, col } => {
                                self.cursors.insert(user_id, (line, col));
                            }
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

    #[test]
    fn test_ide_message_serialization() {
        let msg = IdeMessage::FileUpdate {
            path: "main.rs".to_string(),
            content: "fn main() {}".to_string(),
        };
        let serialized = msg.serialize();
        if let Some(IdeMessage::FileUpdate { path, content }) = IdeMessage::deserialize(&serialized)
        {
            assert_eq!(path, "main.rs");
            assert_eq!(content, "fn main() {}");
        } else {
            panic!("Deserialization failed");
        }

        let msg2 = IdeMessage::CursorMove {
            user_id: 1,
            line: 10,
            col: 5,
        };
        let serialized2 = msg2.serialize();
        if let Some(IdeMessage::CursorMove { user_id, line, col }) =
            IdeMessage::deserialize(&serialized2)
        {
            assert_eq!(user_id, 1);
            assert_eq!(line, 10);
            assert_eq!(col, 5);
        } else {
            panic!("Deserialization failed");
        }
    }

    #[test]
    fn test_ide_client_logic() {
        let mut stack = WasmNetStack::new();
        let mut client = CollaborativeIdeClient::new();
        let addr = IpAddress::v4(192, 168, 1, 4);

        assert!(client.connect_to_ide(&mut stack, addr, 9091).is_ok());

        let handle = *client.connections.get(&addr).unwrap();
        // Since we can't easily mock Established state in smoltcp without real traffic,
        // we will just assert that the socket exists.
        assert!(stack.sockets.get::<TcpSocket>(handle).state() == TcpState::SynSent);
    }
}
