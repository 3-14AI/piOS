#![allow(dead_code)]

#[cfg(feature = "verus")]
use vstd::prelude::*;

#[cfg(feature = "verus")]
verus! {
    pub struct PowerManagement {
        pub state: u8,
    }

    impl PowerManagement {
        pub fn new() -> (p: Self)
            ensures p.state == 0
        {
            PowerManagement { state: 0 }
        }

        pub fn set_state(&mut self, state: u8)
            ensures self.state == state
        {
            self.state = state;
        }
    }
}

#[cfg(not(feature = "verus"))]
pub struct PowerManagement {
    pub state: u8,
}

#[cfg(not(feature = "verus"))]
impl PowerManagement {
    pub fn new() -> Self {
        PowerManagement { state: 0 }
    }

    pub fn set_state(&mut self, state: u8) {
        self.state = state;
    }
}

#[cfg(not(feature = "verus"))]
impl Default for PowerManagement {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "verus"))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_management() {
        let mut pm = PowerManagement::new();
        assert_eq!(pm.state, 0);
        let pm_def = PowerManagement::default();
        assert_eq!(pm_def.state, 0);
        pm.set_state(3);
        assert_eq!(pm.state, 3);
    }
}
