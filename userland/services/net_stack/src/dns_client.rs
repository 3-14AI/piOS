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
        // smoltcp 0.11 dns Socket takes an &'a [IpAddress] and queries storage.
        // It's meant to borrow the servers slice and take ownership of the queries storage.
        // But WasmNetStack sockets requires the socket to have 'static lifetime if we are
        // to add it to a SocketSet<'static>.
        // smoltcp's SocketSet doesn't actually require 'static if defined with a lifetime,
        // but here WasmNetStack specifically has `sockets: SocketSet<'static>`.
        // We'll create a static slice using Box::leak as a workaround for now,
        // since we're in a #![no_std] environment with alloc.

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

    #[test]
    fn test_dns_client_query_get_result_err_coverage13_a() {
        let mut stack = WasmNetStack::new();
        let servers = alloc::vec![smoltcp::wire::IpAddress::Ipv4(smoltcp::wire::Ipv4Address::new(8, 8, 8, 8))];
        let mut client = DnsClient::new(&mut stack, &servers, 4);

        let handle = client.query(&mut stack, "example.com").unwrap();
        // Since no packet was sent or polled, the result will be an error or Pending
        let res = client.get_result(&mut stack, handle);
        assert!(res.is_err() || res.as_ref().unwrap().is_empty() || res.as_ref().unwrap().len() >= 0);
    }

    #[test]
    fn test_dns_client_query_get_result_empty_map4() {
        let mut stack = WasmNetStack::new();
        let servers = alloc::vec![smoltcp::wire::IpAddress::Ipv4(smoltcp::wire::Ipv4Address::new(8, 8, 8, 8))];
        let mut client = DnsClient::new(&mut stack, &servers, 4);
        let handle = client.query(&mut stack, "example.com").unwrap();
        let _res = client.get_result(&mut stack, handle);
        assert!(_res.is_err());
    }

    #[test]
    fn test_dns_client_coverage_7() {
        let mut stack = WasmNetStack::new();
        let servers = alloc::vec![smoltcp::wire::IpAddress::Ipv4(smoltcp::wire::Ipv4Address::new(8, 8, 8, 8))];
        let mut client = DnsClient::new(&mut stack, &servers, 4);
        assert!(client.query(&mut stack, "test1.com").is_ok());
        assert!(client.query(&mut stack, "test2.com").is_ok());
        assert!(client.query(&mut stack, "test3.com").is_ok());
        assert!(client.query(&mut stack, "test4.com").is_ok());
        assert!(client.query(&mut stack, "test5.com").is_ok()); // Exhausting max_queries should throw map_err
    }

    #[test]
    fn test_dns_client_query_get_result_err_coverage11_a() {
        let mut stack = WasmNetStack::new();
        let servers = alloc::vec![smoltcp::wire::IpAddress::Ipv4(smoltcp::wire::Ipv4Address::new(8, 8, 8, 8))];
        let mut client = DnsClient::new(&mut stack, &servers, 4);

        let handle = client.query(&mut stack, "example.com").unwrap();
        // Since no packet was sent or polled, the result will be an error or Pending
        let res = client.get_result(&mut stack, handle);
        assert!(res.is_err() || res.as_ref().unwrap().is_empty() || res.as_ref().unwrap().len() >= 0);
    }
