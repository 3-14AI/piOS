use vstd::prelude::*;

verus! {

pub type PhysAddr = u64;
pub type VirtAddr = u64;

pub struct PageTableEntry {
    pub value: u64,
}

impl PageTableEntry {
    pub open spec fn is_present(self) -> bool {
        (self.value & 0x1) != 0
    }

    pub open spec fn is_writable(self) -> bool {
        (self.value & 0x2) != 0
    }

    pub open spec fn is_user(self) -> bool {
        (self.value & 0x4) != 0
    }

    pub open spec fn is_huge(self) -> bool {
        (self.value & 0x80) != 0
    }

    pub open spec fn address(self) -> PhysAddr {
        self.value & 0x000F_FFFF_FFFF_F000
    }
}

pub open spec fn index_for_level(va: VirtAddr, level: int) -> int {
    if level == 4 {
        ((va >> 39) & 0x1FF) as int
    } else if level == 3 {
        ((va >> 30) & 0x1FF) as int
    } else if level == 2 {
        ((va >> 21) & 0x1FF) as int
    } else {
        ((va >> 12) & 0x1FF) as int
    }
}

pub open spec fn entry_addr(table_base: PhysAddr, index: int) -> PhysAddr {
    (table_base as int + index * 8) as u64
}

// Memory is modeled as a map from physical address (of the entry) to the entry value.
pub type MemState = Map<PhysAddr, PageTableEntry>;

pub open spec fn resolve(mem: MemState, table_base: PhysAddr, va: VirtAddr, level: int) -> Option<PhysAddr>
    decreases level
{
    if level < 1 || level > 4 {
        None
    } else {
        let index = index_for_level(va, level);
        let e_addr = entry_addr(table_base, index);
        if !mem.contains_key(e_addr) {
            None
        } else {
            let entry = mem[e_addr];
            if !entry.is_present() {
                None
            } else if level == 1 || (entry.is_huge() && level > 1) {
                 // Terminal node
                 // For 4K pages (level 1): frame_base + (va & 0xFFF)
                 // For 2M pages (level 2): frame_base + (va & 0x1FFFFF)
                 // For 1G pages (level 3): frame_base + (va & 0x3FFFFFFF)
                 let offset_mask = if level == 1 { 0xFFFu64 } else if level == 2 { 0x1FFFFFu64 } else { 0x3FFFFFFFu64 };
                 let frame_base = entry.address();
                 Some((frame_base + (va & offset_mask)) as u64)
            } else {
                // Non-terminal node
                resolve(mem, entry.address(), va, level - 1)
            }
        }
    }
}

pub open spec fn valid_mapping(mem: MemState, root: PhysAddr, va: VirtAddr, pa: PhysAddr) -> bool {
    resolve(mem, root, va, 4) == Some(pa)
}

}
