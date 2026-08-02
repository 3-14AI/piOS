use smoltcp::socket::dns::Socket as DnsSocket;
use smoltcp::wire::DnsQueryType;
use alloc::vec::Vec;

pub struct DnsResolver {
    servers: Vec<smoltcp::wire::IpAddress>,
}

impl DnsResolver {
    pub fn new(servers: Vec<smoltcp::wire::IpAddress>) -> Self {
        Self { servers }
    }

    pub fn create_socket(&self) -> DnsSocket<'static> {
        let mut servers_vec = Vec::new();
        servers_vec.extend(self.servers.iter().cloned());
        let servers: &'static mut [smoltcp::wire::IpAddress] =
            alloc::boxed::Box::leak(servers_vec.into_boxed_slice());
        DnsSocket::new(servers, alloc::vec![])
    }

    pub fn query(
        &self,
        socket: &mut DnsSocket<'static>,
        context: &mut smoltcp::iface::Context,
        name: &str,
    ) -> Result<smoltcp::socket::dns::QueryHandle, smoltcp::socket::dns::StartQueryError> {
        // Just providing A record request as default query
        socket.start_query(context, name, DnsQueryType::A)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use smoltcp::wire::{IpAddress, Ipv4Address};
    use smoltcp::iface::{Interface, Config, SocketSet};
    use smoltcp::phy::{DeviceCapabilities, Device, Medium, ChecksumCapabilities};
    use smoltcp::time::Instant;

    // A minimal mock device for testing context
    struct MinimalDevice {
        rx_buffer: Vec<u8>,
        tx_buffer: Vec<u8>,
    }

    impl<'a> smoltcp::phy::RxToken for &'a mut MinimalDevice {
        fn consume<R, F>(self, f: F) -> R
        where
            F: FnOnce(&mut [u8]) -> R,
        {
            f(&mut self.rx_buffer)
        }
    }

    impl<'a> smoltcp::phy::TxToken for &'a mut MinimalDevice {
        fn consume<R, F>(self, len: usize, f: F) -> R
        where
            F: FnOnce(&mut [u8]) -> R,
        {
            self.tx_buffer.resize(len, 0);
            f(&mut self.tx_buffer)
        }
    }

    impl Device for MinimalDevice {
        type RxToken<'a> = &'a mut MinimalDevice where Self: 'a;
        type TxToken<'a> = &'a mut MinimalDevice where Self: 'a;

        fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
            None
        }

        fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
            None
        }

        fn capabilities(&self) -> DeviceCapabilities {
            let mut caps = DeviceCapabilities::default();
            caps.max_transmission_unit = 1500;
            caps.medium = Medium::Ethernet;
            caps
        }
    }

    #[test]
    fn test_dns_resolver_socket_and_query() {
        let servers = alloc::vec![IpAddress::Ipv4(Ipv4Address::new(8, 8, 8, 8))];
        let resolver = DnsResolver::new(servers);
        let mut socket = resolver.create_socket();

        let mut device = MinimalDevice {
            rx_buffer: alloc::vec![],
            tx_buffer: alloc::vec![],
        };
        let hardware_addr = smoltcp::wire::HardwareAddress::Ethernet(smoltcp::wire::EthernetAddress([0x02, 0x00, 0x00, 0x00, 0x00, 0x01]));
        let config = Config::new(hardware_addr);
        let mut iface = Interface::new(config, &mut device, Instant::from_millis(0));

        let _ = resolver.query(&mut socket, &mut iface.context(), "example.com");
    }
}
