#![allow(unused_imports)]
use vstd::prelude::*;

verus! {

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Registers {
    pub rbx: u64,
    pub rsp: u64,
    pub rbp: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub rip: u64,
    pub rflags: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThreadState {
    Running,
    Ready,
    Suspended,
    Unused,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TCB {
    pub stack_ptr: u64,
    pub state: ThreadState,
    pub id: u64,
}

impl TCB {
    pub fn new(id: u64, stack_ptr: u64) -> Self {
        TCB {
            stack_ptr,
            state: ThreadState::Unused,
            id,
        }
    }
}

pub ghost struct Thread {
    pub id: u64,
    pub state: ThreadState,
    pub regs: Registers,
}

impl Thread {
    pub open spec fn inv(self, tcb: TCB) -> bool {
        self.id == tcb.id && self.state == tcb.state
    }
}

// Trusted context switch
// Switches from `current` (Running -> Ready) to `next` (Ready -> Running).
#[verifier(external_body)]
pub fn context_switch(
    current_tcb: &mut TCB,
    next_tcb: &mut TCB,
    Ghost(current_thread): Ghost<&mut Thread>,
    Ghost(next_thread): Ghost<&mut Thread>,
)
    requires
        old(current_thread).state == ThreadState::Running,
        old(next_thread).state == ThreadState::Ready,
        old(current_thread).inv(*old(current_tcb)),
        old(next_thread).inv(*old(next_tcb)),
        old(current_tcb).id != old(next_tcb).id,
    ensures
        current_thread.state == ThreadState::Ready,
        next_thread.state == ThreadState::Running,
        current_thread.id == old(current_thread).id,
        next_thread.id == old(next_thread).id,
        current_thread.inv(*current_tcb),
        next_thread.inv(*next_tcb),
{
    unsafe {
        core::arch::asm!(
            "push rbx", "push rbp", "push r12", "push r13", "push r14", "push r15",
            "mov [rdi], rsp", // Save current stack pointer
            "mov rsp, [rsi]", // Load next stack pointer
            "pop r15", "pop r14", "pop r13", "pop r12", "pop rbp", "pop rbx",
            in("rdi") &mut current_tcb.stack_ptr,
            in("rsi") &next_tcb.stack_ptr,
            options(nostack)
        );
    }

    current_tcb.state = ThreadState::Ready;
    next_tcb.state = ThreadState::Running;

    proof {
        current_thread.state = ThreadState::Ready;
        next_thread.state = ThreadState::Running;
    }
}

// Trusted suspend function
// Switches from `current` (Running -> Suspended) to `next` (Ready -> Running).
#[verifier(external_body)]
pub fn suspend_thread(
    current_tcb: &mut TCB,
    next_tcb: &mut TCB,
    Ghost(current_thread): Ghost<&mut Thread>,
    Ghost(next_thread): Ghost<&mut Thread>,
)
    requires
        old(current_thread).state == ThreadState::Running,
        old(next_thread).state == ThreadState::Ready,
        old(current_thread).inv(*old(current_tcb)),
        old(next_thread).inv(*old(next_tcb)),
        old(current_tcb).id != old(next_tcb).id,
    ensures
        current_thread.state == ThreadState::Suspended,
        next_thread.state == ThreadState::Running,
        current_thread.id == old(current_thread).id,
        next_thread.id == old(next_thread).id,
        current_thread.inv(*current_tcb),
        next_thread.inv(*next_tcb),
{
    unsafe {
        // Same context switch mechanism, but state transition is different
        core::arch::asm!(
            "push rbx", "push rbp", "push r12", "push r13", "push r14", "push r15",
            "mov [rdi], rsp",
            "mov rsp, [rsi]",
            "pop r15", "pop r14", "pop r13", "pop r12", "pop rbp", "pop rbx",
            in("rdi") &mut current_tcb.stack_ptr,
            in("rsi") &next_tcb.stack_ptr,
            options(nostack)
        );
    }

    current_tcb.state = ThreadState::Suspended;
    next_tcb.state = ThreadState::Running;

    proof {
        current_thread.state = ThreadState::Suspended;
        next_thread.state = ThreadState::Running;
    }
}

// Trusted wakeup function
// Changes `target` (Suspended -> Ready). No context switch involved here, just state update.
pub fn wake_thread(
    target_tcb: &mut TCB,
    Ghost(target_thread): Ghost<&mut Thread>,
)
    requires
        old(target_thread).state == ThreadState::Suspended,
        old(target_thread).inv(*old(target_tcb)),
    ensures
        target_thread.state == ThreadState::Ready,
        target_thread.id == old(target_thread).id,
        target_thread.inv(*target_tcb),
{
    target_tcb.state = ThreadState::Ready;
    proof {
        target_thread.state = ThreadState::Ready;
    }
}

pub fn test_thread_switching() {
    let mut t1_tcb = TCB { stack_ptr: 0x1000, state: ThreadState::Running, id: 1 };
    let mut t2_tcb = TCB { stack_ptr: 0x2000, state: ThreadState::Ready, id: 2 };
    let mut t3_tcb = TCB { stack_ptr: 0x3000, state: ThreadState::Suspended, id: 3 };

    let ghost mut t1 = Thread {
        id: 1,
        state: ThreadState::Running,
        regs: Registers { rbx: 0, rsp: 0, rbp: 0, r12: 0, r13: 0, r14: 0, r15: 0, rip: 0, rflags: 0 },
    };

    let ghost mut t2 = Thread {
        id: 2,
        state: ThreadState::Ready,
        regs: Registers { rbx: 0, rsp: 0, rbp: 0, r12: 0, r13: 0, r14: 0, r15: 0, rip: 0, rflags: 0 },
    };

    let ghost mut t3 = Thread {
        id: 3,
        state: ThreadState::Suspended,
        regs: Registers { rbx: 0, rsp: 0, rbp: 0, r12: 0, r13: 0, r14: 0, r15: 0, rip: 0, rflags: 0 },
    };

    // 1. Switch Running -> Ready (yield)
    context_switch(&mut t1_tcb, &mut t2_tcb, Ghost(&mut t1), Ghost(&mut t2));
    assert(t1.state == ThreadState::Ready);
    assert(t2.state == ThreadState::Running);

    // 2. Suspend Running -> Suspended
    suspend_thread(&mut t2_tcb, &mut t1_tcb, Ghost(&mut t2), Ghost(&mut t1));
    assert(t2.state == ThreadState::Suspended);
    assert(t1.state == ThreadState::Running);

    // 3. Wake Suspended -> Ready
    wake_thread(&mut t2_tcb, Ghost(&mut t2));
    assert(t2.state == ThreadState::Ready);
}

} // verus!
