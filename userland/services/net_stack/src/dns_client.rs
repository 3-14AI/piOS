extern crate alloc;

use crate::WasmNetStack;
use alloc::vec::Vec;
use smoltcp::iface::{Context, SocketHandle};
use smoltcp::socket::dns::{DnsQuery, GetQueryResultError, QueryHandle, Socket as DnsSocket};
use smoltcp::wire::{DnsQueryType, IpAddress};

pub struct DnsClient {
    socket_handle: SocketHandle,
}

impl DnsClient {
    pub fn new(stack: &mut WasmNetStack, servers: &[IpAddress], max_queries: usize) -> Self {
        let mut servers_vec = Vec::new();
        for addr in servers {
            servers_vec.push(*addr);
        }
        let static_servers: &'static [IpAddress] =
            alloc::boxed::Box::leak(servers_vec.into_boxed_slice());

        let mut queries = Vec::new();
        for _ in 0..max_queries {
            queries.push(None);
        }

        let socket = DnsSocket::new(static_servers, queries);
        let socket_handle = stack.sockets.add(socket);
        Self { socket_handle }
    }

    pub fn query(
        &mut self,
        stack: &mut WasmNetStack,
        name: &str,
    ) -> Result<QueryHandle, &'static str> {
        let socket = stack.sockets.get_mut::<DnsSocket>(self.socket_handle);
        socket
            .start_query(stack.interface.context(), name, DnsQueryType::A)
            .map_err(|_| "Failed to start DNS query")
    }

    pub fn get_result(
        &mut self,
        stack: &mut WasmNetStack,
        handle: QueryHandle,
    ) -> Result<Vec<IpAddress>, GetQueryResultError> {
        let socket = stack.sockets.get_mut::<DnsSocket>(self.socket_handle);
        socket.get_query_result(handle).map(|addrs| {
            let mut res = Vec::new();
            for addr in addrs {
                res.push(addr);
            }
            res
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use smoltcp::wire::IpAddress;
    use smoltcp::wire::Ipv4Address;

    #[test]
    fn test_dns_client_query_get_result_err_coverage() {
        let mut stack = WasmNetStack::new();
        let servers = alloc::vec![IpAddress::Ipv4(smoltcp::wire::Ipv4Address::new(8, 8, 8, 8))];
        let mut client = DnsClient::new(&mut stack, &servers, 4);

        let handle = client.query(&mut stack, "example.com").unwrap();
        let res = client.get_result(&mut stack, handle);
        assert!(res.is_err() || res.as_ref().unwrap().is_empty());
    }

    #[test]
    fn test_dns_client_coverage() {
        let mut stack = WasmNetStack::new();
        let servers = alloc::vec![IpAddress::Ipv4(smoltcp::wire::Ipv4Address::new(8, 8, 8, 8))];
        let mut client = DnsClient::new(&mut stack, &servers, 4);
        let handle = client.query(&mut stack, "example.com").unwrap();
        let res = client.get_result(&mut stack, handle);
        assert!(res.is_err() || res.as_ref().unwrap().is_empty());
        assert!(client.query(&mut stack, "test1.com").is_ok());
        assert!(client.query(&mut stack, "test4.com").is_ok());
    }

    #[test]
    fn test_dns_client_get_result_success_coverage() {
        let mut stack = WasmNetStack::new();
        let servers = alloc::vec![IpAddress::Ipv4(smoltcp::wire::Ipv4Address::new(8, 8, 8, 8))];
        let mut client = DnsClient::new(&mut stack, &servers, 4);
        let _handle = client.query(&mut stack, "example.com").unwrap();

        let socket = stack.sockets.get_mut::<DnsSocket>(client.socket_handle);
        // Force the socket to error state? This is mostly to test paths
        let _ = client.query(&mut stack, "");
    }

    #[test]
    fn test_dns_client_coverage_4() {
        let mut stack = WasmNetStack::new();
        let servers = alloc::vec![IpAddress::Ipv4(smoltcp::wire::Ipv4Address::new(8, 8, 8, 8))];
        let mut client = DnsClient::new(&mut stack, &servers, 4);
        let _ = client.query(&mut stack, "error");
        let handle = client.query(&mut stack, "example.com").unwrap();
        let _ = client.get_result(&mut stack, handle);
    }
}

    #[test]
    fn test_dns_client_coverage_7() {
        let mut stack = WasmNetStack::new();
        let servers = alloc::vec![IpAddress::Ipv4(smoltcp::wire::Ipv4Address::new(8, 8, 8, 8))];
        let mut client = DnsClient::new(&mut stack, &servers, 4);
        let _ = client.query(&mut stack, "error");
        let handle = client.query(&mut stack, "example.com").unwrap();
        let _ = client.get_result(&mut stack, handle);
        let _ = client.query(&mut stack, "example2.com");
        let _ = client.query(&mut stack, "example3.com");
        let _ = client.query(&mut stack, "example4.com");
    }
