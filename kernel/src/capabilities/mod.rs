#![allow(unused_imports)]
extern crate alloc;
use alloc::vec::Vec;

#[cfg(feature = "verus")]
use vstd::prelude::*;

#[cfg(feature = "verus")]
verus! {
    /// Opaque Handle ID
    #[derive(Copy, Clone)]
    pub struct Handle(pub u64);

    /// Abstract capability right
    #[derive(Copy, Clone)]
    pub enum Right {
        Read,
        Write,
        Execute,
    }

    impl Right {
        pub open spec fn eq(self, other: Right) -> bool {
            match self {
                Right::Read => match other { Right::Read => true, _ => false },
                Right::Write => match other { Right::Write => true, _ => false },
                Right::Execute => match other { Right::Execute => true, _ => false },
            }
        }

        pub fn is_eq(&self, other: &Right) -> (res: bool)
            ensures res == self.eq(*other)
        {
            match self {
                Right::Read => match other { Right::Read => true, _ => false },
                Right::Write => match other { Right::Write => true, _ => false },
                Right::Execute => match other { Right::Execute => true, _ => false },
            }
        }
    }

    /// Entry in a capability table mapping a Handle to a resource and its access rights
    #[derive(Copy, Clone)]
    pub struct CapEntry {
        pub handle: Handle,
        pub resource_id: u64, // could be pointer, index, etc.
        pub rights: Right,
    }

    /// Capability Table
    pub struct CapTable {
        pub entries: Vec<CapEntry>,
        pub next_handle: u64,
    }

    impl CapTable {
        pub fn new() -> (c: Self)
            ensures
                c.entries.len() == 0,
                c.next_handle == 1
        {
            CapTable {
                entries: Vec::new(),
                next_handle: 1,
            }
        }

        pub fn mint(&mut self, resource_id: u64, rights: Right) -> (handle: Handle)
            requires old(self).next_handle < 0xffff_ffff_ffff_ffff // To avoid overflow
            ensures
                self.next_handle == old(self).next_handle + 1,
                self.entries.len() == old(self).entries.len() + 1,
                handle.0 == old(self).next_handle,
                self.entries[self.entries.len() - 1].handle.0 == handle.0,
                self.entries[self.entries.len() - 1].resource_id == resource_id,
                self.entries[self.entries.len() - 1].rights.eq(rights)
        {
            let handle = Handle(self.next_handle);
            self.next_handle = self.next_handle + 1;
            let entry = CapEntry { handle, resource_id, rights };
            self.entries.push(entry);
            handle
        }

        pub fn lookup(&self, handle: Handle, expected_right: Right) -> (res: Option<u64>)
            ensures
                match res {
                    Some(rid) => exists|i: int| #![auto] 0 <= i && i < self.entries.len() &&
                        self.entries[i].handle.0 == handle.0 &&
                        self.entries[i].rights.eq(expected_right) &&
                        self.entries[i].resource_id == rid,
                    None => forall|i: int| #![auto] 0 <= i && i < self.entries.len() ==>
                        self.entries[i].handle.0 != handle.0 ||
                        !self.entries[i].rights.eq(expected_right)
                }
        {
            let mut i = 0;
            let mut found: Option<u64> = None;
            while i < self.entries.len()
                invariant
                    0 <= i && i <= self.entries.len(),
                    match found {
                        Some(rid) => exists|j: int| #![auto] 0 <= j && j < i &&
                            self.entries[j].handle.0 == handle.0 &&
                            self.entries[j].rights.eq(expected_right) &&
                            self.entries[j].resource_id == rid,
                        None => forall|j: int| #![auto] 0 <= j && j < i ==>
                            self.entries[j].handle.0 != handle.0 ||
                            !self.entries[j].rights.eq(expected_right)
                    }
                decreases self.entries.len() - i
            {
                if self.entries[i].handle.0 == handle.0 && self.entries[i].rights.is_eq(&expected_right) {
                    found = Some(self.entries[i].resource_id);
                }
                i = i + 1;
            }
            found
        }
    }
}

#[cfg(not(feature = "verus"))]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Handle(pub u64);

#[cfg(not(feature = "verus"))]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Right {
    Read,
    Write,
    Execute,
}

#[cfg(not(feature = "verus"))]
#[derive(Copy, Clone)]
pub struct CapEntry {
    pub handle: Handle,
    pub resource_id: u64,
    pub rights: Right,
}

#[cfg(not(feature = "verus"))]
pub struct CapTable {
    pub entries: alloc::vec::Vec<CapEntry>,
    pub next_handle: u64,
}

#[cfg(not(feature = "verus"))]
impl Default for CapTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "verus"))]
impl CapTable {
    pub fn new() -> Self {
        CapTable {
            entries: alloc::vec::Vec::new(),
            next_handle: 1,
        }
    }

    pub fn mint(&mut self, resource_id: u64, rights: Right) -> Handle {
        let handle = Handle(self.next_handle);
        self.next_handle += 1;
        self.entries.push(CapEntry {
            handle,
            resource_id,
            rights,
        });
        handle
    }

    pub fn lookup(&self, handle: Handle, expected_right: Right) -> Option<u64> {
        for entry in &self.entries {
            if entry.handle.0 == handle.0 && entry.rights == expected_right {
                return Some(entry.resource_id);
            }
        }
        None
    }
}

#[cfg(not(feature = "verus"))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cap_table_mint_and_lookup() {
        let mut table = CapTable::new();

        // Mint capabilities
        let h1 = table.mint(100, Right::Read);
        let h2 = table.mint(200, Right::Write);
        let h3 = table.mint(100, Right::Execute);

        // Lookup existing
        assert_eq!(table.lookup(h1, Right::Read), Some(100));
        assert_eq!(table.lookup(h2, Right::Write), Some(200));
        assert_eq!(table.lookup(h3, Right::Execute), Some(100));

        // Lookup with wrong right
        assert_eq!(table.lookup(h1, Right::Write), None);
        assert_eq!(table.lookup(h2, Right::Read), None);

        // Lookup non-existent handle
        assert_eq!(table.lookup(Handle(999), Right::Read), None);
    }
}
