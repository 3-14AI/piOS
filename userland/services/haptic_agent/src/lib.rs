#![no_std]

pub struct HapticAgent {
    pub haptic_active: bool,
}

impl Default for HapticAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl HapticAgent {
    pub fn new() -> Self {
        Self {
            haptic_active: false,
        }
    }

    pub fn trigger_feedback(&mut self, event_type: &str) {
        if event_type == "compile_failure" || event_type == "malware_detection" {
            self.haptic_active = true;
        }
    }

    pub fn reset_feedback(&mut self) {
        self.haptic_active = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_haptic_feedback() {
        let mut agent = HapticAgent::new();
        assert_eq!(agent.haptic_active, false);

        agent.trigger_feedback("compile_failure");
        assert_eq!(agent.haptic_active, true);

        agent.reset_feedback();
        assert_eq!(agent.haptic_active, false);
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[cfg(not(test))]
#[global_allocator]
static ALLOCATOR: pios_common::DummyAllocator = pios_common::DummyAllocator;
