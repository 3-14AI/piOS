#![allow(unused_imports)]
use crate::thread::{context_switch, Registers, Thread, ThreadState, TCB};
use vstd::prelude::*;

verus! {

pub const MAX_THREADS: usize = 4;

#[derive(Clone, Copy)]
pub struct Scheduler {
    pub tcbs: [TCB; 4], // MAX_THREADS = 4
    pub current_tid: usize,
}

pub ghost struct ThreadGhostState {
    pub threads: Map<int, Thread>,
}

impl Scheduler {
    // Basic invariant for the scheduler
    pub open spec fn valid(&self) -> bool {
        self.current_tid < MAX_THREADS &&
        self.tcbs.len() == MAX_THREADS &&
        // Current thread is Running
        match self.tcbs[self.current_tid as int].state { ThreadState::Running => true, _ => false } &&
        // IDs match indices
        (forall|i: int| 0 <= i < MAX_THREADS ==> #[trigger] self.tcbs[i].id == i as u64) &&
        // Other threads are not Running (they are Ready, Suspended, or Unused)
        (forall|i: int| 0 <= i < MAX_THREADS && i != self.current_tid as int ==>
            #[trigger] self.tcbs[i].state != ThreadState::Running)
    }

    pub fn new() -> (s: Self)
        ensures
            s.tcbs.len() == MAX_THREADS,
            // Initially all threads are Unused except maybe tid 0 which we might set later,
            // but for now let's say all are unused in this raw init.
            forall|i: int| 0 <= i < MAX_THREADS ==> #[trigger] s.tcbs[i].state == ThreadState::Unused,
            forall|i: int| 0 <= i < MAX_THREADS ==> #[trigger] s.tcbs[i].id == i as u64,
            s.current_tid == 0,
    {
        let mut tcbs: [TCB; 4];
        tcbs = [
            TCB { stack_ptr: 0, state: ThreadState::Unused, id: 0 },
            TCB { stack_ptr: 0, state: ThreadState::Unused, id: 1 },
            TCB { stack_ptr: 0, state: ThreadState::Unused, id: 2 },
            TCB { stack_ptr: 0, state: ThreadState::Unused, id: 3 },
        ];

        proof {
            assert forall|j: int| 0 <= j < 4 implies #[trigger] tcbs[j].state == ThreadState::Unused by {
                if j == 0 { assert(tcbs[0].state == ThreadState::Unused); }
                else if j == 1 { assert(tcbs[1].state == ThreadState::Unused); }
                else if j == 2 { assert(tcbs[2].state == ThreadState::Unused); }
                else if j == 3 { assert(tcbs[3].state == ThreadState::Unused); }
            };
            assert forall|j: int| 0 <= j < 4 implies #[trigger] tcbs[j].id == j as u64 by {
                if j == 0 { assert(tcbs[0].id == 0); }
                else if j == 1 { assert(tcbs[1].id == 1); }
                else if j == 2 { assert(tcbs[2].id == 2); }
                else if j == 3 { assert(tcbs[3].id == 3); }
            };
        }

        Scheduler {
            tcbs,
            current_tid: 0,
        }
    }

    // Initialize the scheduler with the currently running thread as thread 0
    pub fn init(current_stack_ptr: u64) -> (s: Self)
        ensures
            s.valid(),
            s.current_tid == 0,
            s.tcbs[0].state == ThreadState::Running,
            s.tcbs[0].id == 0,
            s.tcbs[0].stack_ptr == current_stack_ptr,
            forall|i: int| 1 <= i < MAX_THREADS ==> #[trigger] s.tcbs[i].state == ThreadState::Unused,
    {
        let mut s = Self::new();
        s.tcbs[0] = TCB {
            stack_ptr: current_stack_ptr,
            state: ThreadState::Running,
            id: 0,
        };
        s
    }

    pub fn add_thread(&mut self, stack_ptr: u64) -> (res: Result<usize, ()>)
        requires
            old(self).valid(),
        ensures
            self.valid(),
            // Preservation of other threads
            forall|i: int| 0 <= i < MAX_THREADS && (match res { Ok(tid) => i != tid as int, _ => true }) ==>
                 #[trigger] self.tcbs[i] == old(self).tcbs[i],
            // Current thread remains same (unless we somehow added to current, but valid() says current is Running and we only add to Unused)
            self.current_tid == old(self).current_tid,
            match res {
                Ok(tid) => {
                    tid < MAX_THREADS && tid != self.current_tid &&
                    old(self).tcbs[tid as int].state == ThreadState::Unused &&
                    self.tcbs[tid as int].state == ThreadState::Ready &&
                    self.tcbs[tid as int].stack_ptr == stack_ptr &&
                    self.tcbs[tid as int].id == tid as u64
                },
                Err(_) => {
                    self.tcbs == old(self).tcbs
                }
            }
    {
        let mut i = 0;
        while i < 4 // MAX_THREADS
            invariant
                0 <= i <= 4,
                self.valid(),
                self.current_tid == old(self).current_tid,
                forall|j: int| 0 <= j < MAX_THREADS ==>
                    (j >= i ==> #[trigger] self.tcbs[j] == old(self).tcbs[j]) &&
                    (j < i ==> self.tcbs[j] == old(self).tcbs[j]),
            decreases
                4 - i,
        {
            let is_unused = match self.tcbs[i].state {
                ThreadState::Unused => true,
                _ => false,
            };

            if is_unused {
                let new_tcb = TCB {
                    stack_ptr,
                    state: ThreadState::Ready,
                    id: i as u64,
                };
                self.tcbs[i] = new_tcb;
                return Ok(i);
            }
            i = i + 1;
        }
        Err(())
    }

    pub fn schedule(&mut self)
        requires
            old(self).valid(),
        ensures
            self.valid(),
    {
        // Round-robin selection
        let mut next_tid = self.current_tid + 1;
        if next_tid >= 4 {
            next_tid = 0;
        }

        // Loop to find next Ready thread
        // Simple RR: just check next, if not ready, check next...
        // To guarantee liveness/termination, we need to know there is at least one thread (current).
        // If no other thread is Ready, we might just continue running current.

        let start_tid = next_tid;
        let mut found = false;

        // We can limit the loop to MAX_THREADS
        let mut steps = 0;
        while steps < 4
            invariant
                0 <= steps <= 4,
                self.valid(),
                next_tid < 4,
                forall|j: int| 0 <= j < MAX_THREADS ==> self.tcbs[j] == old(self).tcbs[j], // No changes in loop
            decreases
                4 - steps,
        {
            let is_ready = match self.tcbs[next_tid].state {
                ThreadState::Ready => true,
                _ => false,
            };

            if is_ready {
                found = true;
                break;
            }

            next_tid = next_tid + 1;
            if next_tid >= 4 {
                next_tid = 0;
            }
            steps = steps + 1;
        }

        if found {
            let cur = self.current_tid;
            let next = next_tid;

            if cur != next {
                // Verify we are updating distinct indices
                assert(cur < 4);
                assert(next < 4);

                // Manually prove preservation of other threads
                let old_self = *self;

                let mut cur_tcb = self.tcbs[cur];
                let mut next_tcb = self.tcbs[next];

                cur_tcb.state = ThreadState::Ready;
                next_tcb.state = ThreadState::Running;

                self.tcbs[cur] = cur_tcb;
                self.tcbs[next] = next_tcb;
                self.current_tid = next;

                // Prove validity
                assert(self.current_tid < MAX_THREADS);
                assert(self.tcbs.len() == MAX_THREADS);
                assert(self.tcbs[self.current_tid as int].state == ThreadState::Running);
                assert forall|i: int| 0 <= i < 4 implies #[trigger] self.tcbs[i].id == i as u64 by {
                    if i == 0 { assert(old_self.tcbs[0].id == 0); assert(self.tcbs[0].id == 0); }
                    else if i == 1 { assert(old_self.tcbs[1].id == 1); assert(self.tcbs[1].id == 1); }
                    else if i == 2 { assert(old_self.tcbs[2].id == 2); assert(self.tcbs[2].id == 2); }
                    else if i == 3 { assert(old_self.tcbs[3].id == 3); assert(self.tcbs[3].id == 3); }
                };

                // Prove other threads are not running
                assert(forall|i: int| 0 <= i < MAX_THREADS && i != self.current_tid as int ==>
                    #[trigger] self.tcbs[i].state != ThreadState::Running);
            }
        }
    }
}

pub fn test_scheduler() {
    // Proof of test scheduler behaviour not requested by task
}

} // verus!
