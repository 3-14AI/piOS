extern crate alloc;

use alloc::vec::Vec;
use smoltcp::iface::{SocketHandle, Context};
use smoltcp::socket::dns::{Socket as DnsSocket, QueryHandle, GetQueryResultError, DnsQuery};
use smoltcp::wire::{IpAddress, DnsQueryType};
use crate::WasmNetStack;

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
        let static_servers: &'static [IpAddress] = alloc::boxed::Box::leak(servers_vec.into_boxed_slice());

        let mut queries = Vec::new();
        for _ in 0..max_queries {
            queries.push(None);
        }

        let socket = DnsSocket::new(static_servers, queries);
        let socket_handle = stack.sockets.add(socket);
        Self { socket_handle }
    }

    pub fn query(&mut self, stack: &mut WasmNetStack, name: &str) -> Result<QueryHandle, &'static str> {
        let socket = stack.sockets.get_mut::<DnsSocket>(self.socket_handle);
        socket.start_query(stack.interface.context(), name, DnsQueryType::A).map_err(|_| "Failed to start DNS query")
    }

    pub fn get_result(&mut self, stack: &mut WasmNetStack, handle: QueryHandle) -> Result<Vec<IpAddress>, GetQueryResultError> {
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
