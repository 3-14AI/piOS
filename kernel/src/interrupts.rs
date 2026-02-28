#![allow(unused_imports)]
use vstd::prelude::*;

verus! {

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum InterruptState {
    Unmasked,
    Masked,
    Pending,
    InService,
}

pub const MAX_INTERRUPTS: usize = 256;

#[derive(Clone, Copy)]
pub struct InterruptController {
    pub states: [InterruptState; 256], // MAX_INTERRUPTS = 256
}

impl InterruptController {
    pub open spec fn valid(&self) -> bool {
        self.states.len() == MAX_INTERRUPTS
    }

    pub fn new() -> (s: Self)
        ensures
            s.valid(),
            forall|i: int| 0 <= i < MAX_INTERRUPTS ==> s.states[i] == InterruptState::Masked,
    {
        let mut states = [InterruptState::Masked; 256];
        let mut i = 0;
        while i < 256
            invariant
                0 <= i <= 256,
                states.len() == 256,
                forall|j: int| 0 <= j < i ==> states[j] == InterruptState::Masked,
            decreases
                256 - i,
        {
            states[i] = InterruptState::Masked;
            i = i + 1;
        }

        InterruptController { states }
    }

    pub fn unmask(&mut self, irq: usize)
        requires
            old(self).valid(),
            irq < MAX_INTERRUPTS,
            old(self).states[irq as int] == InterruptState::Masked,
        ensures
            self.valid(),
            self.states[irq as int] == InterruptState::Unmasked,
            forall|j: int| 0 <= j < MAX_INTERRUPTS && j != irq as int ==>
                self.states[j] == old(self).states[j],
    {
        self.states[irq] = InterruptState::Unmasked;
    }

    pub fn mask(&mut self, irq: usize)
        requires
            old(self).valid(),
            irq < MAX_INTERRUPTS,
            old(self).states[irq as int] == InterruptState::Unmasked,
        ensures
            self.valid(),
            self.states[irq as int] == InterruptState::Masked,
            forall|j: int| 0 <= j < MAX_INTERRUPTS && j != irq as int ==>
                self.states[j] == old(self).states[j],
    {
        self.states[irq] = InterruptState::Masked;
    }

    pub fn trigger(&mut self, irq: usize)
        requires
            old(self).valid(),
            irq < MAX_INTERRUPTS,
            old(self).states[irq as int] == InterruptState::Unmasked,
        ensures
            self.valid(),
            self.states[irq as int] == InterruptState::Pending,
            forall|j: int| 0 <= j < MAX_INTERRUPTS && j != irq as int ==>
                self.states[j] == old(self).states[j],
    {
        self.states[irq] = InterruptState::Pending;
    }

    pub fn ack(&mut self, irq: usize)
        requires
            old(self).valid(),
            irq < MAX_INTERRUPTS,
            old(self).states[irq as int] == InterruptState::Pending,
        ensures
            self.valid(),
            self.states[irq as int] == InterruptState::InService,
            forall|j: int| 0 <= j < MAX_INTERRUPTS && j != irq as int ==>
                self.states[j] == old(self).states[j],
    {
        self.states[irq] = InterruptState::InService;
    }

    pub fn eoi(&mut self, irq: usize)
        requires
            old(self).valid(),
            irq < MAX_INTERRUPTS,
            old(self).states[irq as int] == InterruptState::InService,
        ensures
            self.valid(),
            self.states[irq as int] == InterruptState::Unmasked,
            forall|j: int| 0 <= j < MAX_INTERRUPTS && j != irq as int ==>
                self.states[j] == old(self).states[j],
    {
        self.states[irq] = InterruptState::Unmasked;
    }
}

pub fn test_interrupts() {
    let mut controller = InterruptController::new();
    assert(controller.valid());
    assert(controller.states[42] == InterruptState::Masked);

    controller.unmask(42);
    assert(controller.valid());
    assert(controller.states[42] == InterruptState::Unmasked);

    controller.trigger(42);
    assert(controller.valid());
    assert(controller.states[42] == InterruptState::Pending);

    controller.ack(42);
    assert(controller.valid());
    assert(controller.states[42] == InterruptState::InService);

    controller.eoi(42);
    assert(controller.valid());
    assert(controller.states[42] == InterruptState::Unmasked);

    controller.mask(42);
    assert(controller.valid());
    assert(controller.states[42] == InterruptState::Masked);
}

} // verus!
