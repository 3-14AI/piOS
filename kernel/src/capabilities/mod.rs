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
        pub uid: u32,
        pub gid: u32,
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

        pub fn mint(&mut self, resource_id: u64, rights: Right, uid: u32, gid: u32) -> (handle: Handle)
            requires old(self).next_handle < 0xffff_ffff_ffff_ffff // To avoid overflow
            ensures
                self.next_handle == old(self).next_handle + 1,
                self.entries.len() == old(self).entries.len() + 1,
                handle.0 == old(self).next_handle,
                self.entries[self.entries.len() - 1].handle.0 == handle.0,
                self.entries[self.entries.len() - 1].resource_id == resource_id,
                self.entries[self.entries.len() - 1].rights.eq(rights),
                self.entries[self.entries.len() - 1].uid == uid,
                self.entries[self.entries.len() - 1].gid == gid
        {
            let handle = Handle(self.next_handle);
            self.next_handle = self.next_handle + 1;
            let entry = CapEntry { handle, resource_id, rights, uid, gid };
            self.entries.push(entry);
            handle
        }

        pub fn lookup(&self, handle: Handle, expected_right: Right, uid: u32, gid: u32) -> (res: Option<u64>)
            ensures
                match res {
                    Some(rid) => exists|i: int| #![auto] 0 <= i && i < self.entries.len() &&
                        self.entries[i].handle.0 == handle.0 &&
                        self.entries[i].rights.eq(expected_right) &&
                        (self.entries[i].uid == uid || self.entries[i].gid == gid || uid == 0) &&
                        self.entries[i].resource_id == rid,
                    None => forall|i: int| #![auto] 0 <= i && i < self.entries.len() ==>
                        self.entries[i].handle.0 != handle.0 ||
                        !self.entries[i].rights.eq(expected_right) ||
                        (self.entries[i].uid != uid && self.entries[i].gid != gid && uid != 0)
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
                            (self.entries[j].uid == uid || self.entries[j].gid == gid || uid == 0) &&
                            self.entries[j].resource_id == rid,
                        None => forall|j: int| #![auto] 0 <= j && j < i ==>
                            self.entries[j].handle.0 != handle.0 ||
                            !self.entries[j].rights.eq(expected_right) ||
                            (self.entries[j].uid != uid && self.entries[j].gid != gid && uid != 0)
                    }
                decreases self.entries.len() - i
            {
                if self.entries[i].handle.0 == handle.0 && self.entries[i].rights.is_eq(&expected_right) {
                    if self.entries[i].uid == uid || self.entries[i].gid == gid || uid == 0 {
                        found = Some(self.entries[i].resource_id);
                    }
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
    pub uid: u32,
    pub gid: u32,
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

    pub fn mint(&mut self, resource_id: u64, rights: Right, uid: u32, gid: u32) -> Handle {
        let handle = Handle(self.next_handle);
        self.next_handle += 1;
        self.entries.push(CapEntry {
            handle,
            resource_id,
            rights,
            uid,
            gid,
        });
        handle
    }

    pub fn lookup(&self, handle: Handle, expected_right: Right, uid: u32, gid: u32) -> Option<u64> {
        for entry in &self.entries {
            if entry.handle.0 == handle.0
                && entry.rights == expected_right
                && (entry.uid == uid || entry.gid == gid || uid == 0)
            {
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
        let h1 = table.mint(100, Right::Read, 1000, 1000);
        let h2 = table.mint(200, Right::Write, 1000, 1000);
        let h3 = table.mint(100, Right::Execute, 1001, 1001);

        // Lookup existing
        assert_eq!(table.lookup(h1, Right::Read, 1000, 1000), Some(100));
        assert_eq!(table.lookup(h2, Right::Write, 1000, 1000), Some(200));
        assert_eq!(table.lookup(h3, Right::Execute, 1001, 1001), Some(100));

        // Lookup with wrong right
        assert_eq!(table.lookup(h1, Right::Write, 1000, 1000), None);
        assert_eq!(table.lookup(h2, Right::Read, 1000, 1000), None);

        // Lookup non-existent handle
        assert_eq!(table.lookup(Handle(999), Right::Read, 1000, 1000), None);

        // Lookup with root user
        assert_eq!(table.lookup(h1, Right::Read, 0, 0), Some(100));
        // Lookup with wrong user
        assert_eq!(table.lookup(h1, Right::Read, 1002, 1002), None);
    }

    #[test]
    fn test_multi_user_sessions_and_sudo() {
        let mut table = CapTable::new();

        // Users:
        // Alice: UID 1000, GID 1000
        // Bob: UID 1001, GID 1000 (same group as Alice)
        // Charlie: UID 1002, GID 1002 (different group)
        // Root: UID 0, GID 0

        let file1_alice_write = table.mint(10, Right::Write, 1000, 9999);
        let file1_group_read = table.mint(10, Right::Read, 1000, 1000);

        let file2_bob_read = table.mint(20, Right::Read, 1001, 1001);

        // 1. Session Isolation: Charlie cannot access Bob private file
        assert_eq!(table.lookup(file2_bob_read, Right::Read, 1002, 1002), None);

        // 2. Session Isolation: Alice cannot access Bob private file
        assert_eq!(table.lookup(file2_bob_read, Right::Read, 1000, 1000), None);

        // 3. Owner Access: Bob can access his private file
        assert_eq!(
            table.lookup(file2_bob_read, Right::Read, 1001, 1001),
            Some(20)
        );

        // 4. Group Access: Bob can read Alice file because he shares GID 1000
        assert_eq!(
            table.lookup(file1_group_read, Right::Read, 1001, 1000),
            Some(10)
        );

        // 5. Session Isolation: Bob cannot write to Alice file
        assert_eq!(
            table.lookup(file1_alice_write, Right::Write, 1001, 1000),
            None
        );

        // 6. Owner Access: Alice can write to her file
        assert_eq!(
            table.lookup(file1_alice_write, Right::Write, 1000, 1000),
            Some(10)
        );

        // 7. Sudo-analog: Root (UID 0) can access ANY capability
        assert_eq!(table.lookup(file2_bob_read, Right::Read, 0, 0), Some(20));
        assert_eq!(
            table.lookup(file1_alice_write, Right::Write, 0, 0),
            Some(10)
        );
    }
}
