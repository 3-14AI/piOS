#![no_std]
#![allow(unused)]

extern crate alloc;

use alloc::vec::Vec;
use smoltcp::iface::{Interface, SocketSet};
use smoltcp::phy::{ChecksumCapabilities, Device, DeviceCapabilities, Medium};
use smoltcp::socket::tcp::{
    Socket as TcpSocket, SocketBuffer as TcpSocketBuffer, State as TcpState,
};
use smoltcp::time::Instant;
use smoltcp::wire::{EthernetAddress, HardwareAddress, IpAddress, IpCidr, Ipv4Address};

#[cfg(feature = "verus")]
use vstd::prelude::*;

// A mock network device for testing
pub struct MockDevice {
    rx_buffer: Vec<u8>,
    tx_buffer: Vec<u8>,
}

impl Default for MockDevice {
    fn default() -> Self {
        Self::new()
    }
}

impl MockDevice {
    pub fn new() -> Self {
        Self {
            rx_buffer: Vec::new(),
            tx_buffer: Vec::new(),
        }
    }
}

impl Device for MockDevice {
    type RxToken<'a>
        = MockRxToken<'a>
    where
        Self: 'a;
    type TxToken<'a>
        = MockTxToken<'a>
    where
        Self: 'a;

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        None
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        None
    }

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.max_transmission_unit = 1500;
        caps.max_burst_size = Some(1);
        caps.medium = Medium::Ethernet;
        caps.checksum = ChecksumCapabilities::ignored();
        caps
    }
}

pub struct MockRxToken<'a> {
    buffer: &'a mut Vec<u8>,
}

impl<'a> smoltcp::phy::RxToken for MockRxToken<'a> {
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        f(self.buffer.as_mut_slice())
    }
}

pub struct MockTxToken<'a> {
    buffer: &'a mut Vec<u8>,
}

impl<'a> smoltcp::phy::TxToken for MockTxToken<'a> {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        self.buffer.resize(len, 0);
        f(self.buffer.as_mut_slice())
    }
}

#[cfg(feature = "verus")]
verus! {
    // Abstract state of a TCP connection
    pub enum AbsTcpState {
        Closed,
        Listen,
        SynSent,
        SynReceived,
        Established,
        FinWait1,
        FinWait2,
        CloseWait,
        Closing,
        LastAck,
        TimeWait,
    }

    // A purely abstract model of the smoltcp TCP state machine
    pub struct TcpStateMachine {
        pub state: AbsTcpState,
    }

    impl TcpStateMachine {
        pub open spec fn init() -> Self {
            Self { state: AbsTcpState::Closed }
        }

        pub open spec fn listen(self) -> Self
            recommends self.state == AbsTcpState::Closed
        {
            Self { state: AbsTcpState::Listen }
        }

        pub open spec fn connect(self) -> Self
            recommends self.state == AbsTcpState::Closed
        {
            Self { state: AbsTcpState::SynSent }
        }

        pub open spec fn rcv_syn(self) -> Self
            recommends self.state == AbsTcpState::Listen
        {
            Self { state: AbsTcpState::SynReceived }
        }

        pub open spec fn rcv_synack(self) -> Self
            recommends self.state == AbsTcpState::SynSent
        {
            Self { state: AbsTcpState::Established }
        }

        pub open spec fn rcv_ack_of_syn(self) -> Self
            recommends self.state == AbsTcpState::SynReceived
        {
            Self { state: AbsTcpState::Established }
        }

        pub open spec fn close(self) -> Self
            recommends self.state == AbsTcpState::Established
        {
            Self { state: AbsTcpState::FinWait1 }
        }

        pub open spec fn rcv_fin(self) -> Self
            recommends self.state == AbsTcpState::Established
        {
            Self { state: AbsTcpState::CloseWait }
        }

        pub open spec fn rcv_ack_of_fin(self) -> Self
            recommends self.state == AbsTcpState::FinWait1
        {
            Self { state: AbsTcpState::FinWait2 }
        }

        pub open spec fn rcv_fin_and_ack(self) -> Self
            recommends self.state == AbsTcpState::FinWait1
        {
            Self { state: AbsTcpState::TimeWait }
        }
    }

    // Function to map real smoltcp state to our abstract state
    #[verifier(external)]
    pub fn map_smoltcp_state(real_state: TcpState) -> AbsTcpState {
        match real_state {
            TcpState::Closed => AbsTcpState::Closed,
            TcpState::Listen => AbsTcpState::Listen,
            TcpState::SynSent => AbsTcpState::SynSent,
            TcpState::SynReceived => AbsTcpState::SynReceived,
            TcpState::Established => AbsTcpState::Established,
            TcpState::FinWait1 => AbsTcpState::FinWait1,
            TcpState::FinWait2 => AbsTcpState::FinWait2,
            TcpState::CloseWait => AbsTcpState::CloseWait,
            TcpState::Closing => AbsTcpState::Closing,
            TcpState::LastAck => AbsTcpState::LastAck,
            TcpState::TimeWait => AbsTcpState::TimeWait,
        }
    }
}

pub struct WasmNetStack {
    device: MockDevice,
    interface: Interface,
    sockets: SocketSet<'static>,
}

impl Default for WasmNetStack {
    fn default() -> Self {
        Self::new()
    }
}

impl WasmNetStack {
    pub fn new() -> Self {
        let mut device = MockDevice::new();
        let hardware_addr =
            HardwareAddress::Ethernet(EthernetAddress([0x02, 0x00, 0x00, 0x00, 0x00, 0x01]));
        let config = smoltcp::iface::Config::new(hardware_addr);
        let mut interface = Interface::new(config, &mut device, Instant::from_millis(0));

        interface.update_ip_addrs(|ip_addrs| {
            ip_addrs
                .push(IpCidr::new(IpAddress::v4(192, 168, 1, 1), 24))
                .unwrap();
        });

        Self {
            device,
            interface,
            sockets: SocketSet::new(Vec::new()),
        }
    }

    pub fn add_tcp_socket(&mut self) -> smoltcp::iface::SocketHandle {
        let rx_buffer = TcpSocketBuffer::new(alloc::vec![0; 1024]);
        let tx_buffer = TcpSocketBuffer::new(alloc::vec![0; 1024]);
        let socket = TcpSocket::new(rx_buffer, tx_buffer);
        self.sockets.add(socket)
    }

    pub fn poll(&mut self, time: Instant) {
        self.interface
            .poll(time, &mut self.device, &mut self.sockets);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_net_stack() {
        let mut stack = WasmNetStack::new();
        let handle = stack.add_tcp_socket();
        assert!(stack.sockets.get::<TcpSocket>(handle).state() == TcpState::Closed);
    }

    #[test]
    fn test_tcp_state_transitions() {
        let mut stack = WasmNetStack::new();
        let handle = stack.add_tcp_socket();

        let socket = stack.sockets.get_mut::<TcpSocket>(handle);
        assert_eq!(socket.state(), TcpState::Closed);

        socket.listen(80).unwrap();
        assert_eq!(socket.state(), TcpState::Listen);
    }
}
