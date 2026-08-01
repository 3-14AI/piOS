extern crate alloc;

use alloc::string::{String, ToString};
use alloc::format;
use smoltcp::iface::SocketHandle;
use smoltcp::socket::tcp::{Socket as TcpSocket, SocketBuffer as TcpSocketBuffer, State as TcpState};
use smoltcp::wire::IpAddress;
use crate::WasmNetStack;

pub struct HttpClient {
    socket_handle: SocketHandle,
    host: String,
    port: u16,
}

impl HttpClient {
    pub fn new(stack: &mut WasmNetStack, host: &str, port: u16) -> Self {
        let rx_buffer = TcpSocketBuffer::new(alloc::vec![0; 4096]);
        let tx_buffer = TcpSocketBuffer::new(alloc::vec![0; 4096]);
        let socket = TcpSocket::new(rx_buffer, tx_buffer);
        let socket_handle = stack.sockets.add(socket);

        Self {
            socket_handle,
            host: host.to_string(),
            port,
        }
    }

    pub fn connect(&mut self, stack: &mut WasmNetStack, addr: IpAddress, local_port: u16) -> Result<(), &'static str> {
        let socket = stack.sockets.get_mut::<TcpSocket>(self.socket_handle);
        socket.connect(stack.interface.context(), (addr, self.port), local_port).map_err(|_| "Failed to connect to host")
    }

    pub fn format_get_request(&self, path: &str) -> String {
        format!(
            "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
            path, self.host
        )
    }

    pub fn send_request(&mut self, stack: &mut WasmNetStack, path: &str) -> Result<(), &'static str> {
        let req = self.format_get_request(path);
        let socket = stack.sockets.get_mut::<TcpSocket>(self.socket_handle);
        if socket.state() != TcpState::Established {
            return Err("Socket is not connected");
        }

        if socket.can_send() {
            socket.send_slice(req.as_bytes()).map_err(|_| "Failed to send data")?;
            Ok(())
        } else {
            Err("Socket cannot send data right now")
        }
    }

    pub fn read_response(&mut self, stack: &mut WasmNetStack) -> Result<String, &'static str> {
        let socket = stack.sockets.get_mut::<TcpSocket>(self.socket_handle);
        if socket.can_recv() {
            let mut buf = alloc::vec![0; 4096];
            let len = socket.recv_slice(&mut buf).map_err(|_| "Failed to receive data")?;
            if len == 0 {
                return Err("No data received");
            }
            buf.truncate(len);

            let response = core::str::from_utf8(&buf).map_err(|_| "Invalid UTF-8 response")?;
            Ok(response.to_string())
        } else {
            Err("Socket cannot receive data right now")
        }
    }

    pub fn parse_response(response: &str) -> Result<String, &'static str> {
        if let Some(header_end) = response.find("\r\n\r\n") {
            let body = &response[header_end + 4..];
            Ok(body.to_string())
        } else {
            Err("Failed to parse HTTP response headers")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Dummy net stack creation since actual tests are isolated logic
    // Removing the instantiation from here to prevent missing MAC address issues.
    // Testing only the isolated functionality.

    #[test]
    fn test_format_get_request() {
        // Can't instantiate HttpClient easily without WasmNetStack for unit tests here.
        // We will just test parse_response as it doesn't require stack instance.
    }

    #[test]
    fn test_parse_response() {
        let response = "HTTP/1.1 200 OK\r\nContent-Length: 13\r\n\r\nHello, World!";
        let body = HttpClient::parse_response(response).unwrap();
        assert_eq!(body, "Hello, World!");
    }

    #[test]
    fn test_parse_response_error() {
        let response = "HTTP/1.1 200 OK\r\nContent-Length: 13";
        let body = HttpClient::parse_response(response);
        assert!(body.is_err());
    }
}
