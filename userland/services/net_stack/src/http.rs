use smoltcp::socket::tcp::{Socket as TcpSocket, SocketBuffer as TcpSocketBuffer};

pub struct HttpClient {}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpClient {
    pub fn new() -> Self {
        Self {}
    }

    pub fn create_socket(&self) -> TcpSocket<'static> {
        let rx_buffer = TcpSocketBuffer::new(alloc::vec![0; 1024]);
        let tx_buffer = TcpSocketBuffer::new(alloc::vec![0; 1024]);
        TcpSocket::new(rx_buffer, tx_buffer)
    }

    pub fn construct_get_request(host: &str, path: &str) -> alloc::string::String {
        alloc::format!(
            "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
            path,
            host
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_client_socket() {
        let client = HttpClient::default();
        let _socket = client.create_socket();
    }

    #[test]
    fn test_construct_get_request() {
        let req = HttpClient::construct_get_request("example.com", "/index.html");
        assert_eq!(
            req,
            "GET /index.html HTTP/1.1\r\nHost: example.com\r\nConnection: close\r\n\r\n"
        );
    }
}
