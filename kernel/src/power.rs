#![allow(dead_code)]

#[cfg(feature = "verus")]
use vstd::prelude::*;

#[cfg(feature = "verus")]
verus! {
    pub struct PowerManagement {
        pub state: u8,
        pub cpu_freq_mhz: u32,
    }

    impl PowerManagement {
        pub fn new() -> (p: Self)
            ensures p.state == 0 && p.cpu_freq_mhz == 2000
        {
            PowerManagement { state: 0, cpu_freq_mhz: 2000 }
        }

        pub fn set_state(&mut self, state: u8)
            ensures self.state == state
        {
            self.state = state;
        }

        pub fn suspend(&mut self)
            ensures self.state == 3
        {
            self.state = 3;
        }

        pub fn hibernate(&mut self)
            ensures self.state == 4
        {
            self.state = 4;
        }

        pub fn sleep(&mut self)
            ensures self.state == 1
        {
            self.state = 1;
        }

        pub fn set_cpu_freq(&mut self, freq_mhz: u32)
            ensures self.cpu_freq_mhz == freq_mhz
        {
            self.cpu_freq_mhz = freq_mhz;
        }
    }
}

#[cfg(not(feature = "verus"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SState {
    S0Working = 0,
    S1Sleep = 1,
    S2Sleep = 2,
    S3Standby = 3,
    S4Hibernate = 4,
    S5SoftOff = 5,
}

#[cfg(not(feature = "verus"))]
impl From<u8> for SState {
    fn from(val: u8) -> Self {
        match val {
            0 => SState::S0Working,
            1 => SState::S1Sleep,
            2 => SState::S2Sleep,
            3 => SState::S3Standby,
            4 => SState::S4Hibernate,
            5 => SState::S5SoftOff,
            _ => SState::S0Working,
        }
    }
}

#[cfg(not(feature = "verus"))]
pub struct PowerManagement {
    pub state: SState,
    pub cpu_freq_mhz: u32,
}

#[cfg(not(feature = "verus"))]
impl PowerManagement {
    pub fn new() -> Self {
        PowerManagement { state: SState::S0Working, cpu_freq_mhz: 2000 }
    }

    pub fn set_state(&mut self, state: SState) {
        self.state = state;
    }

    pub fn suspend(&mut self) {
        self.state = SState::S3Standby;
    }

    pub fn hibernate(&mut self) {
        self.state = SState::S4Hibernate;
    }

    pub fn sleep(&mut self) {
        self.state = SState::S1Sleep;
    }

    pub fn set_cpu_freq(&mut self, freq_mhz: u32) {
        self.cpu_freq_mhz = freq_mhz;
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
        assert_eq!(pm.state, SState::S0Working);
        assert_eq!(pm.cpu_freq_mhz, 2000);

        let pm_def = PowerManagement::default();
        assert_eq!(pm_def.state, SState::S0Working);

        pm.set_state(SState::S3Standby);
        assert_eq!(pm.state, SState::S3Standby);

        pm.set_cpu_freq(1000);
        assert_eq!(pm.cpu_freq_mhz, 1000);

        pm.suspend();
        assert_eq!(pm.state, SState::S3Standby);

        pm.hibernate();
        assert_eq!(pm.state, SState::S4Hibernate);

        pm.sleep();
        assert_eq!(pm.state, SState::S1Sleep);

        assert_eq!(SState::from(3), SState::S3Standby);
        assert_eq!(SState::from(99), SState::S0Working);
    }
}
