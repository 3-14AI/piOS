#![allow(unused_imports)]
#![allow(dead_code)]

extern crate alloc;
use alloc::vec::Vec;

pub struct SleepManager {
    pub display_on: bool,
    pub network_on: bool,
}

impl Default for SleepManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SleepManager {
    pub fn new() -> Self {
        Self {
            display_on: true,
            network_on: true,
        }
    }

    pub fn manage_sleep_states(
        &mut self,
        user_active: bool,
        battery_level: u8,
    ) -> Result<(), &'static str> {
        // Sleep state management heuristics
        if !user_active {
            if battery_level < 20 {
                self.display_on = false;
                self.network_on = false; // Aggressive power saving
            } else {
                self.display_on = false; // Turn off display only
            }
        } else {
            self.display_on = true;
            self.network_on = true;
        }

        #[cfg(target_arch = "wasm32")]
        unsafe {
            let display_state = if self.display_on { 1 } else { 0 };
            sys_set_component_sleep_state(1, display_state); // ID 1 for display

            let network_state = if self.network_on { 1 } else { 0 };
            sys_set_component_sleep_state(2, network_state); // ID 2 for network
        }

        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "wasi_snapshot_preview1")]
extern "C" {
    fn sys_set_component_sleep_state(component_id: i32, state: i32) -> i32;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manage_sleep_states() {
        let mut mgr = SleepManager::new();

        // Active user, good battery -> everything on
        assert!(mgr.manage_sleep_states(true, 100).is_ok());
        assert!(mgr.display_on);
        assert!(mgr.network_on);

        // Inactive user, good battery -> display off, network on
        assert!(mgr.manage_sleep_states(false, 50).is_ok());
        assert!(!mgr.display_on);
        assert!(mgr.network_on);

        // Inactive user, low battery -> aggressive power saving
        assert!(mgr.manage_sleep_states(false, 10).is_ok());
        assert!(!mgr.display_on);
        assert!(!mgr.network_on);
    }
}
