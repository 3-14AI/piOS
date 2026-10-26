use vstd::prelude::*;

verus! {

pub const PAGE_SIZE: u64 = 4096;
pub const MAX_PAGES: usize = 4096; // 16MB managed memory

pub struct PhysicalMemoryManager {
    // True = Allocated, False = Free
    map: [bool; MAX_PAGES],
}

impl PhysicalMemoryManager {
    pub open spec fn valid_page_index(i: int) -> bool {
        0 <= i < MAX_PAGES
    }

    pub open spec fn index_to_addr(i: int) -> u64 {
        (i * (PAGE_SIZE as int)) as u64
    }

    pub open spec fn addr_to_index(addr: u64) -> int {
        addr as int / (PAGE_SIZE as int)
    }

    // Abstract model: The set of free page indices
    pub closed spec fn free_pages(&self) -> Set<int> {
        Set::new(|i: int| Self::valid_page_index(i) && !self.map[i])
    }

    pub fn new() -> (s: Self)
        ensures s.free_pages() == Set::new(|i: int| Self::valid_page_index(i))
    {
        let mut map = [false; MAX_PAGES];

        // Loop to prove initialization property to Verus
        let mut i = 0;
        while i < MAX_PAGES
            invariant
                0 <= i <= MAX_PAGES,
                forall|j: int| 0 <= j < i ==> map[j] == false,
                // We also need to know that map length is MAX_PAGES, which is implicit for array type
            decreases
                MAX_PAGES - i,
        {
            map[i] = false;
            i = i + 1;
        }

        PhysicalMemoryManager { map }
    }

    pub fn alloc(&mut self) -> (res: Option<u64>)
        ensures
            match res {
                Some(addr) => {
                    let idx = Self::addr_to_index(addr);
                    old(self).free_pages().contains(idx) &&
                    !self.free_pages().contains(idx) &&
                    self.free_pages() == old(self).free_pages().remove(idx)
                },
                None => self.free_pages() == old(self).free_pages(),
            }
    {
        let mut i = 0;
        while i < MAX_PAGES
            invariant
                0 <= i <= MAX_PAGES,
                // If we are here, all previous pages were allocated (true)
                forall|j: int| 0 <= j < i ==> self.map[j] == true,
                // The map hasn't changed
                self.map == old(self).map,
            decreases
                MAX_PAGES - i,
        {
            if !self.map[i] {
                self.map[i] = true;
                let addr = (i as u64) * PAGE_SIZE;
                assert(Self::addr_to_index(addr) == i as int);
                return Some(addr);
            }
            i = i + 1;
        }
        None
    }

    pub fn free(&mut self, addr: u64)
        requires
            addr % PAGE_SIZE == 0,
            Self::valid_page_index(Self::addr_to_index(addr)),
            !old(self).free_pages().contains(Self::addr_to_index(addr)), // Must be allocated
        ensures
            self.free_pages() == old(self).free_pages().insert(Self::addr_to_index(addr)),
    {
        let idx = (addr / PAGE_SIZE) as usize;
        self.map[idx] = false;
    }
}

} // verus!

#[cfg(feature = "verus")]
impl PhysicalMemoryManager {
    /// Analyzes the memory map to determine an external fragmentation score.
    /// Returns a value between 0.0 (perfectly contiguous) and 1.0 (highly fragmented).
    pub fn analyze_fragmentation(&self) -> f32 {
        let mut free_chunks = 0;
        let mut total_free = 0;
        let mut in_free_chunk = false;

        for i in 0..MAX_PAGES {
            if !self.map[i] {
                total_free += 1;
                if !in_free_chunk {
                    free_chunks += 1;
                    in_free_chunk = true;
                }
            } else {
                in_free_chunk = false;
            }
        }

        if total_free == 0 || free_chunks <= 1 {
            return 0.0;
        }

        // Score based on number of chunks relative to total free pages
        // More chunks per free page = higher fragmentation
        (free_chunks as f32) / (total_free as f32)
    }

    /// Attempts to simulate defragmentation by consolidating free pages.
    /// In a real system, this would involve updating page tables and physically moving data.
    /// Here we just log/simulate the effort if fragmentation is high.
    pub fn defragment(&mut self) -> bool {
        // True physical memory defragmentation is highly complex and requires migrating
        // live pages, which implies mapping changes and updating page tables.
        // For the scope of WP-198, we only compute the score and trigger
        // an OS-level intent (or log) rather than corrupting the current memory map
        // by moving physical tracking blindly.
        let score = self.analyze_fragmentation();
        if score > 0.5 {
            // Trigger an alert or signal for the high-level agent to attempt a
            // coordinated memory squeeze.
            return true;
        }
        false
    }
}

verus! {

} // verus!

// Global instance for the allocator (Unverified static access for now)
// In a real verified system, this would be protected by a verified spinlock.
// We put this outside verus! block but guarded by the feature because PhysicalMemoryManager depends on verus feature?
// Actually, PhysicalMemoryManager is defined inside verus! which is guarded by cfg(feature="verus") in lib.rs.
// So this whole file is guarded. We don't need extra guard here if the file is only included when feature is on.
// But to be safe and explicit:

#[cfg(feature = "verus")]
pub static mut GLOBAL_PMM: Option<PhysicalMemoryManager> = None;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pmm_alloc_and_free() {
        let mut pmm = PhysicalMemoryManager::new();
        let addr1 = pmm.alloc().unwrap();
        assert_eq!(addr1, 0);

        let addr2 = pmm.alloc().unwrap();
        assert_eq!(addr2, 4096);

        pmm.free(addr1);

        let addr3 = pmm.alloc().unwrap();
        assert_eq!(addr3, 0); // Should be reused
    }

    #[test]
    fn test_pmm_out_of_memory() {
        let mut pmm = PhysicalMemoryManager::new();
        for _ in 0..MAX_PAGES {
            assert!(pmm.alloc().is_some());
        }
        assert!(pmm.alloc().is_none());
    }
}
