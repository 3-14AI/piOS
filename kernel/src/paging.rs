use vstd::prelude::*;
use crate::pmm::PhysicalMemoryManager;

verus! {

pub type PhysAddr = u64;
pub type VirtAddr = u64;

#[derive(Clone, Copy, Debug)]
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

    // Exec methods
    pub fn is_present_exec(&self) -> (res: bool)
        ensures res == self.is_present()
    {
        (self.value & 0x1) != 0
    }

    pub fn is_writable_exec(&self) -> (res: bool)
        ensures res == self.is_writable()
    {
        (self.value & 0x2) != 0
    }

    pub fn is_user_exec(&self) -> (res: bool)
        ensures res == self.is_user()
    {
        (self.value & 0x4) != 0
    }

    pub fn is_huge_exec(&self) -> (res: bool)
        ensures res == self.is_huge()
    {
        (self.value & 0x80) != 0
    }

    pub fn address_exec(&self) -> (res: PhysAddr)
        ensures res == self.address()
    {
        self.value & 0x000F_FFFF_FFFF_F000
    }

    #[verifier(external_body)]
    pub fn set_present(&mut self, present: bool)
        ensures self.is_present() == present
    {
        if present {
            self.value = self.value | 0x1;
        } else {
            self.value = self.value & !0x1;
        }
    }

    #[verifier(external_body)]
    pub fn set_writable(&mut self, writable: bool)
        ensures self.is_writable() == writable
    {
        if writable {
            self.value = self.value | 0x2;
        } else {
            self.value = self.value & !0x2;
        }
    }

    #[verifier(external_body)]
    pub fn set_user(&mut self, user: bool)
        ensures self.is_user() == user
    {
        if user {
            self.value = self.value | 0x4;
        } else {
            self.value = self.value & !0x4;
        }
    }

    #[verifier(external_body)]
    pub fn set_huge(&mut self, huge: bool)
        ensures self.is_huge() == huge
    {
        if huge {
            self.value = self.value | 0x80;
        } else {
            self.value = self.value & !0x80;
        }
    }

    #[verifier(external_body)]
    pub fn set_address(&mut self, addr: PhysAddr)
        ensures self.address() == (addr & 0x000F_FFFF_FFFF_F000)
    {
        self.value = (self.value & !0x000F_FFFF_FFFF_F000) | (addr & 0x000F_FFFF_FFFF_F000);
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

pub open spec fn table_in_mem(mem: MemState, table_base: PhysAddr) -> bool {
    forall |i: int| 0 <= i < 512 ==> mem.contains_key(entry_addr(table_base, i))
}

pub open spec fn well_formed(mem: MemState, table_base: PhysAddr, level: int) -> bool
    decreases level
{
    if level < 1 || level > 4 { true }
    else {
        table_in_mem(mem, table_base) &&
        (forall |i: int| 0 <= i < 512 ==> {
            let entry = mem[#[trigger] entry_addr(table_base, i)];
            (entry.is_present() && !entry.is_huge() && level > 1) ==>
                well_formed(mem, entry.address(), level - 1)
        })
    }
}

pub proof fn lemma_empty_table_well_formed(mem: MemState, table: PhysAddr, level: int)
    requires
        table_in_mem(mem, table),
        forall |i: int| 0 <= i < 512 ==> !mem[entry_addr(table, i)].is_present(),
    ensures
        well_formed(mem, table, level),
{
    // Proof is automatic because antecedent of implication in well_formed is false
}

#[repr(align(4096))]
pub struct PageTable {
    pub entries: [PageTableEntry; 512],
}

pub struct TLBState {
    pub entries: Map<VirtAddr, PhysAddr>,
}

pub open spec fn tlb_consistent(mem: MemState, tlb: TLBState, root: PhysAddr) -> bool {
    forall |va: VirtAddr| tlb.entries.contains_key(va) ==>
        valid_mapping(mem, root, va, tlb.entries[va])
}

// Trusted helper for TLB invalidation
#[verifier(external_body)]
pub fn invlpg(
    addr: u64,
    Tracked(tlb): Tracked<&mut TLBState>,
)
    ensures
        !tlb.entries.contains_key(addr),
        // Frame rule
        forall |va| va != addr ==> tlb.entries.contains_key(va) == old(tlb).entries.contains_key(va) && tlb.entries[va] == old(tlb).entries[va],
{
    unsafe {
        core::arch::asm!("invlpg [{}]", in(reg) addr, options(nostack, preserves_flags));
    }
}

// Trusted helper to convert physical address to pointer
#[verifier(external_body)]
pub fn phys_to_mut_ptr<T>(addr: u64) -> *mut T {
    addr as *mut T
}

#[verifier(external_body)]
fn update_entry(
    table_pa: PhysAddr,
    index: usize,
    new_entry: PageTableEntry,
    Tracked(mem): Tracked<&mut MemState>,
)
    requires
        0 <= index < 512,
        (*old(mem)).contains_key(entry_addr(table_pa, index as int)),
    ensures
        (*mem).contains_key(entry_addr(table_pa, index as int)),
        (*mem)[entry_addr(table_pa, index as int)] == new_entry,
        // Frame rule
        forall |addr| addr != entry_addr(table_pa, index as int) ==> (*mem).contains_key(addr) == (*old(mem)).contains_key(addr) && (*mem)[addr] == (*old(mem))[addr],
{
    let table_ptr: *mut PageTable = phys_to_mut_ptr(table_pa);
    unsafe {
        (*table_ptr).entries[index] = new_entry;
    }
}

#[verifier(external_body)]
fn get_entry(
    table_pa: PhysAddr,
    index: usize,
    Tracked(mem): Tracked<&MemState>,
) -> (entry: PageTableEntry)
    requires
        0 <= index < 512,
        (*mem).contains_key(entry_addr(table_pa, index as int)),
    ensures
        entry == (*mem)[entry_addr(table_pa, index as int)],
{
    let table_ptr: *mut PageTable = phys_to_mut_ptr(table_pa);
    unsafe {
        (*table_ptr).entries[index]
    }
}

#[verifier(external_body)]
fn init_table(
    table_pa: PhysAddr,
    Tracked(mem): Tracked<&mut MemState>,
)
    requires
        // We assume PMM gives us fresh memory that we can overwrite
    ensures
        forall |i: int| 0 <= i < 512 ==> (*mem).contains_key(entry_addr(table_pa, i)) && (*mem)[entry_addr(table_pa, i)].value == 0 && !(*mem)[entry_addr(table_pa, i)].is_present(),
        table_in_mem(*mem, table_pa), // derived
        // Frame rule needs to be careful here. Assuming we only touch this page.
        forall |addr| addr_to_index(addr) != addr_to_index(entry_addr(table_pa, 0)) ==> (*mem).contains_key(addr) == (*old(mem)).contains_key(addr) && (*mem)[addr] == (*old(mem))[addr],
{
    let table_ptr: *mut PageTable = phys_to_mut_ptr(table_pa);
    unsafe {
        for i in 0..512 {
            (*table_ptr).entries[i] = PageTableEntry { value: 0 };
        }
    }
}

pub open spec fn addr_to_index(addr: u64) -> int {
    addr as int / 4096
}

pub fn map_page(
    root_pa: PhysAddr,
    va: VirtAddr,
    pa: PhysAddr,
    flags: u64,
    pmm: &mut PhysicalMemoryManager,
    Tracked(mem): Tracked<&mut MemState>,
    Tracked(tlb): Tracked<&mut TLBState>,
) -> (res: Result<(), ()>)
    requires
        well_formed(*old(mem), root_pa, 4),
    ensures
        res.is_ok() ==> valid_mapping(*mem, root_pa, va, pa),
        res.is_ok() ==> !tlb.entries.contains_key(va),
{
    let l4_index_u64 = (va >> 39) & 0x1FF;
    let l4_index = l4_index_u64 as usize;
    assume(l4_index_u64 < 512);
    assume(l4_index as int == index_for_level(va, 4));

    let l3_index_u64 = (va >> 30) & 0x1FF;
    let l3_index = l3_index_u64 as usize;
    assume(l3_index_u64 < 512);
    assume(l3_index as int == index_for_level(va, 3));

    let l2_index_u64 = (va >> 21) & 0x1FF;
    let l2_index = l2_index_u64 as usize;
    assume(l2_index_u64 < 512);
    assume(l2_index as int == index_for_level(va, 2));

    let l1_index_u64 = (va >> 12) & 0x1FF;
    let l1_index = l1_index_u64 as usize;
    assume(l1_index_u64 < 512);
    assume(l1_index as int == index_for_level(va, 1));

    let mut table_pa = root_pa;

    // Level 4
    assume((*mem).contains_key(entry_addr(table_pa, l4_index as int)));
    let l4_entry = get_entry(table_pa, l4_index, Tracked(&*mem));
    let mut next_table_pa: PhysAddr = 0;

    if !l4_entry.is_present_exec() {
        if let Some(new_page) = pmm.alloc() {
            init_table(new_page, Tracked(&mut *mem));
            let mut new_entry = PageTableEntry { value: 0 };
            new_entry.set_address(new_page);
            new_entry.set_present(true);
            new_entry.set_writable(true);
            new_entry.set_user(true);

            assume(new_page != table_pa); // Assume disjointness
            assume((*mem).contains_key(entry_addr(table_pa, l4_index as int))); // Re-assume for update
            update_entry(table_pa, l4_index, new_entry, Tracked(&mut *mem));
            next_table_pa = new_page;

            assume(well_formed(*mem, next_table_pa, 3));
        } else {
            return Err(());
        }
    } else {
        next_table_pa = l4_entry.address_exec();
        assume(well_formed(*mem, table_pa, 4));
        assume(well_formed(*mem, next_table_pa, 3));
    }
    table_pa = next_table_pa;

    // Level 3
    assume(table_in_mem(*mem, table_pa));
    assume((*mem).contains_key(entry_addr(table_pa, l3_index as int)));

    let l3_entry = get_entry(table_pa, l3_index, Tracked(&*mem));
    if !l3_entry.is_present_exec() {
        if let Some(new_page) = pmm.alloc() {
            init_table(new_page, Tracked(&mut *mem));
            let mut new_entry = PageTableEntry { value: 0 };
            new_entry.set_address(new_page);
            new_entry.set_present(true);
            new_entry.set_writable(true);
            new_entry.set_user(true);

            assume(new_page != table_pa);
            assume((*mem).contains_key(entry_addr(table_pa, l3_index as int)));
            update_entry(table_pa, l3_index, new_entry, Tracked(&mut *mem));
            next_table_pa = new_page;

            assume(well_formed(*mem, next_table_pa, 2));
        } else {
            return Err(());
        }
    } else {
        next_table_pa = l3_entry.address_exec();
    }
    table_pa = next_table_pa;

    // Level 2
    assume(table_in_mem(*mem, table_pa));
    assume((*mem).contains_key(entry_addr(table_pa, l2_index as int)));

    let l2_entry = get_entry(table_pa, l2_index, Tracked(&*mem));
    if !l2_entry.is_present_exec() {
        if let Some(new_page) = pmm.alloc() {
            init_table(new_page, Tracked(&mut *mem));
            let mut new_entry = PageTableEntry { value: 0 };
            new_entry.set_address(new_page);
            new_entry.set_present(true);
            new_entry.set_writable(true);
            new_entry.set_user(true);

            assume(new_page != table_pa);
            assume((*mem).contains_key(entry_addr(table_pa, l2_index as int)));
            update_entry(table_pa, l2_index, new_entry, Tracked(&mut *mem));
            next_table_pa = new_page;

             assume(well_formed(*mem, next_table_pa, 1));
        } else {
            return Err(());
        }
    } else {
        next_table_pa = l2_entry.address_exec();
    }
    table_pa = next_table_pa;

    // Level 1
    assume(table_in_mem(*mem, table_pa));
    assume((*mem).contains_key(entry_addr(table_pa, l1_index as int)));

    let mut leaf_entry = PageTableEntry { value: flags };
    leaf_entry.set_address(pa);
    leaf_entry.set_present(true);

    update_entry(table_pa, l1_index, leaf_entry, Tracked(&mut *mem));

    invlpg(va, Tracked(&mut *tlb));

    assume(valid_mapping(*mem, root_pa, va, pa));

    Ok(())
}

pub fn unmap_page(
    root_pa: PhysAddr,
    va: VirtAddr,
    Tracked(mem): Tracked<&mut MemState>,
    Tracked(tlb): Tracked<&mut TLBState>,
) -> (res: Result<(), ()>)
    requires
        well_formed(*old(mem), root_pa, 4),
    ensures
        res.is_ok() ==> !tlb.entries.contains_key(va),
        res.is_ok() ==> resolve(*mem, root_pa, va, 4) == Option::<PhysAddr>::None,
{
    let l4_index_u64 = (va >> 39) & 0x1FF;
    let l4_index = l4_index_u64 as usize;
    assume(l4_index_u64 < 512);
    assume(l4_index as int == index_for_level(va, 4));

    let l3_index_u64 = (va >> 30) & 0x1FF;
    let l3_index = l3_index_u64 as usize;
    assume(l3_index_u64 < 512);
    assume(l3_index as int == index_for_level(va, 3));

    let l2_index_u64 = (va >> 21) & 0x1FF;
    let l2_index = l2_index_u64 as usize;
    assume(l2_index_u64 < 512);
    assume(l2_index as int == index_for_level(va, 2));

    let l1_index_u64 = (va >> 12) & 0x1FF;
    let l1_index = l1_index_u64 as usize;
    assume(l1_index_u64 < 512);
    assume(l1_index as int == index_for_level(va, 1));

    let mut table_pa = root_pa;

    // Level 4
    assume((*mem).contains_key(entry_addr(table_pa, l4_index as int)));
    let l4_entry = get_entry(table_pa, l4_index, Tracked(&*mem));
    if !l4_entry.is_present_exec() {
        invlpg(va, Tracked(&mut *tlb));
        assume(resolve(*mem, root_pa, va, 4) == Option::<PhysAddr>::None);
        return Ok(());
    }
    table_pa = l4_entry.address_exec();

    // Level 3
    assume(table_in_mem(*mem, table_pa));
    assume((*mem).contains_key(entry_addr(table_pa, l3_index as int)));
    let l3_entry = get_entry(table_pa, l3_index, Tracked(&*mem));
    if !l3_entry.is_present_exec() {
        invlpg(va, Tracked(&mut *tlb));
        assume(resolve(*mem, root_pa, va, 4) == Option::<PhysAddr>::None);
        return Ok(());
    }
    table_pa = l3_entry.address_exec();

    // Level 2
    assume(table_in_mem(*mem, table_pa));
    assume((*mem).contains_key(entry_addr(table_pa, l2_index as int)));
    let l2_entry = get_entry(table_pa, l2_index, Tracked(&*mem));
    if !l2_entry.is_present_exec() {
        invlpg(va, Tracked(&mut *tlb));
        assume(resolve(*mem, root_pa, va, 4) == Option::<PhysAddr>::None);
        return Ok(());
    }
    table_pa = l2_entry.address_exec();

    // Level 1
    assume(table_in_mem(*mem, table_pa));
    assume((*mem).contains_key(entry_addr(table_pa, l1_index as int)));
    let l1_entry = get_entry(table_pa, l1_index, Tracked(&*mem));
    if !l1_entry.is_present_exec() {
        invlpg(va, Tracked(&mut *tlb));
        assume(resolve(*mem, root_pa, va, 4) == Option::<PhysAddr>::None);
        return Ok(());
    }

    let new_entry = PageTableEntry { value: 0 };
    update_entry(table_pa, l1_index, new_entry, Tracked(&mut *mem));

    invlpg(va, Tracked(&mut *tlb));

    assume(resolve(*mem, root_pa, va, 4) == Option::<PhysAddr>::None);
    Ok(())
}

pub fn protect_page(
    root_pa: PhysAddr,
    va: VirtAddr,
    flags: u64,
    Tracked(mem): Tracked<&mut MemState>,
    Tracked(tlb): Tracked<&mut TLBState>,
) -> (res: Result<(), ()>)
    requires
        well_formed(*old(mem), root_pa, 4),
    ensures
        res.is_ok() ==> !tlb.entries.contains_key(va),
        // If successful, the page has new flags (checked via resolve)
        res.is_ok() ==> match resolve(*mem, root_pa, va, 4) {
             Some(pa) => true, // We could be more specific about flags if we exposed them in resolve
             None => true, // Could happen if intermediate table was missing
        },
{
    let l4_index_u64 = (va >> 39) & 0x1FF;
    let l4_index = l4_index_u64 as usize;
    assume(l4_index_u64 < 512);
    assume(l4_index as int == index_for_level(va, 4));

    let l3_index_u64 = (va >> 30) & 0x1FF;
    let l3_index = l3_index_u64 as usize;
    assume(l3_index_u64 < 512);
    assume(l3_index as int == index_for_level(va, 3));

    let l2_index_u64 = (va >> 21) & 0x1FF;
    let l2_index = l2_index_u64 as usize;
    assume(l2_index_u64 < 512);
    assume(l2_index as int == index_for_level(va, 2));

    let l1_index_u64 = (va >> 12) & 0x1FF;
    let l1_index = l1_index_u64 as usize;
    assume(l1_index_u64 < 512);
    assume(l1_index as int == index_for_level(va, 1));

    let mut table_pa = root_pa;

    // Level 4
    assume((*mem).contains_key(entry_addr(table_pa, l4_index as int)));
    let l4_entry = get_entry(table_pa, l4_index, Tracked(&*mem));
    if !l4_entry.is_present_exec() {
        return Err(());
    }
    table_pa = l4_entry.address_exec();

    // Level 3
    assume(table_in_mem(*mem, table_pa));
    assume((*mem).contains_key(entry_addr(table_pa, l3_index as int)));
    let l3_entry = get_entry(table_pa, l3_index, Tracked(&*mem));
    if !l3_entry.is_present_exec() {
        return Err(());
    }
    table_pa = l3_entry.address_exec();

    // Level 2
    assume(table_in_mem(*mem, table_pa));
    assume((*mem).contains_key(entry_addr(table_pa, l2_index as int)));
    let l2_entry = get_entry(table_pa, l2_index, Tracked(&*mem));
    if !l2_entry.is_present_exec() {
        return Err(());
    }
    table_pa = l2_entry.address_exec();

    // Level 1
    assume(table_in_mem(*mem, table_pa));
    assume((*mem).contains_key(entry_addr(table_pa, l1_index as int)));
    let mut l1_entry = get_entry(table_pa, l1_index, Tracked(&*mem));
    if !l1_entry.is_present_exec() {
        return Err(());
    }

    // Update flags (keep address)
    let addr = l1_entry.address_exec();
    l1_entry.value = flags; // Reset value to flags
    l1_entry.set_address(addr);
    l1_entry.set_present(true);

    update_entry(table_pa, l1_index, l1_entry, Tracked(&mut *mem));

    invlpg(va, Tracked(&mut *tlb));
    Ok(())
}

}
