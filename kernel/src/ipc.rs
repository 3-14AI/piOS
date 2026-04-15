#![allow(unused_imports)]

#[cfg(feature = "verus")]
use vstd::prelude::*;

#[cfg(feature = "verus")]
verus! {

/// Synchronous Rendezvous Channel for IPC (Message Passing)
///
/// This implements WP-009: "Механизм IPC (Message Passing). Реализовать синхронную передачу
/// сообщений (Rendezvous). Доказать атомарность передачи владения данными (Linear Types logic)
/// для исключения гонок данных."
///
/// We use Rust's Option type and linear typing to represent that memory ownership
/// is completely transferred from sender to receiver.
/// In Verus, we prove that:
/// - A successful send moves the data into the channel, leaving it full.
/// - A successful recv takes the data out, leaving it empty.
/// - `data` being consumed avoids data races between sender and receiver.

pub struct RendezvousChannel<T> {
    pub data: Option<T>,
}

impl<T> RendezvousChannel<T> {
    pub open spec fn is_empty(&self) -> bool {
        self.data.is_none()
    }

    pub open spec fn is_full(&self) -> bool {
        self.data.is_some()
    }

    pub fn new() -> (c: Self)
        ensures c.is_empty()
    {
        RendezvousChannel { data: None }
    }

    /// Tries to send a message.
    /// Transfers ownership of `val` into the channel.
    pub fn try_send(&mut self, val: T) -> (res: Result<(), T>)
        ensures
            res.is_ok() ==> old(self).is_empty() && self.is_full() && self.data == Some(val),
            res.is_err() ==> !old(self).is_empty() && self.data == old(self).data,
    {
        let is_some = match &self.data {
            Some(_) => true,
            None => false,
        };
        if is_some {
            Err(val)
        } else {
            self.data = Some(val);
            Ok(())
        }
    }

    /// Tries to receive a message.
    /// Transfers ownership of the inner value out of the channel.
    pub fn try_recv(&mut self) -> (res: Option<T>)
        ensures
            res.is_some() ==> old(self).is_full() && self.is_empty() && res == old(self).data,
            res.is_none() ==> old(self).is_empty() && self.is_empty(),
    {
        let is_none = match &self.data {
            Some(_) => false,
            None => true,
        };
        if is_none {
            None
        } else {
            self.data.take()
        }
    }
}

pub fn test_rendezvous() {
    let mut channel = RendezvousChannel::<u64>::new();
    assert(channel.is_empty());

    let val: u64 = 42;
    match channel.try_send(val) {
        Ok(()) => {
            assert(channel.is_full());
            assert(channel.data == Some(42u64));
        },
        Err(_) => {
            assert(false);
        }
    }

    match channel.try_recv() {
        Some(v) => {
            assert(v == 42u64);
            assert(channel.is_empty());
        },
        None => {
            assert(false);
        }
    }
}

} // verus!

#[cfg(not(feature = "verus"))]
pub struct RendezvousChannel<T> {
    pub data: Option<T>,
}

#[cfg(not(feature = "verus"))]
impl<T> RendezvousChannel<T> {
    pub fn new() -> Self {
        Self { data: None }
    }

    pub fn try_send(&mut self, val: T) -> Result<(), T> {
        if self.data.is_some() {
            Err(val)
        } else {
            self.data = Some(val);
            Ok(())
        }
    }

    pub fn try_recv(&mut self) -> Option<T> {
        self.data.take()
    }
}

#[cfg(not(feature = "verus"))]
impl<T> Default for RendezvousChannel<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "verus"))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipc_rendezvous() {
        let mut channel = RendezvousChannel::<u64>::new();
        assert!(channel.try_recv().is_none());
        assert!(channel.try_send(42).is_ok());
        assert!(channel.try_send(43).is_err());
        assert_eq!(channel.try_recv(), Some(42));
        assert!(channel.try_recv().is_none());
    }
}
