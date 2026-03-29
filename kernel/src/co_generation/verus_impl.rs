use vstd::prelude::*;

verus! {

pub struct CoGenerationFramework {
    pub is_running: bool,
    pub tests_generated: u64,
}

impl CoGenerationFramework {
    pub closed spec fn default_tests() -> u64 {
        0
    }

    pub fn new() -> (c: Self)
        ensures
            c.is_running == false,
            c.tests_generated == Self::default_tests(),
    {
        Self {
            is_running: false,
            tests_generated: 0,
        }
    }

    pub fn generate_test(&mut self)
        requires
            old(self).is_running == true,
            old(self).tests_generated < 1000,
        ensures
            self.is_running == true,
            self.tests_generated == old(self).tests_generated + 1,
    {
        self.tests_generated = self.tests_generated + 1;
    }

    pub fn start(&mut self)
        requires
            old(self).is_running == false,
        ensures
            self.is_running == true,
            self.tests_generated == old(self).tests_generated,
    {
        self.is_running = true;
    }

    pub fn stop(&mut self)
        requires
            old(self).is_running == true,
        ensures
            self.is_running == false,
            self.tests_generated == old(self).tests_generated,
    {
        self.is_running = false;
    }
}

} // verus!
