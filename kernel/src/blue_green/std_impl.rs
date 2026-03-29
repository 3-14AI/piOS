#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Component {
    pub version: u32,
    pub data: u32,
}

pub struct BlueGreenManager {
    pub primary_component: Component,
    pub shadow_component: Option<Component>,
}

impl BlueGreenManager {
    pub fn new(primary: Component) -> Self {
        Self {
            primary_component: primary,
            shadow_component: None,
        }
    }

    pub fn update_shadow(&mut self, new_shadow: Component) {
        if new_shadow.version > self.primary_component.version {
            self.shadow_component = Some(new_shadow);
        }
    }

    pub fn execute_and_compare(&self, input: u32) -> bool {
        if let Some(shadow) = self.shadow_component {
            let primary_result = self.primary_component.data.wrapping_add(input);
            let shadow_result = shadow.data.wrapping_add(input);
            primary_result == shadow_result
        } else {
            false
        }
    }

    pub fn promote_shadow(&mut self) {
        if let Some(shadow) = self.shadow_component {
            self.primary_component = shadow;
            self.shadow_component = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blue_green_lifecycle() {
        let primary = Component {
            version: 1,
            data: 100,
        };
        let mut manager = BlueGreenManager::new(primary);

        assert_eq!(manager.primary_component.version, 1);
        assert!(manager.shadow_component.is_none());

        // Update shadow
        let shadow = Component {
            version: 2,
            data: 100,
        };
        manager.update_shadow(shadow);

        assert!(manager.shadow_component.is_some());
        assert_eq!(manager.shadow_component.unwrap().version, 2);

        // Execute and compare (should match since data is the same)
        assert!(manager.execute_and_compare(50));

        // Promote shadow
        manager.promote_shadow();
        assert_eq!(manager.primary_component.version, 2);
        assert!(manager.shadow_component.is_none());
    }

    #[test]
    fn test_execute_and_compare_mismatch() {
        let primary = Component {
            version: 1,
            data: 100,
        };
        let mut manager = BlueGreenManager::new(primary);

        let shadow = Component {
            version: 2,
            data: 200,
        }; // Different data
        manager.update_shadow(shadow);

        assert!(!manager.execute_and_compare(50));
    }

    #[test]
    fn test_update_shadow_lower_version() {
        let primary = Component {
            version: 2,
            data: 100,
        };
        let mut manager = BlueGreenManager::new(primary);

        let shadow = Component {
            version: 1,
            data: 100,
        }; // Lower version
        manager.update_shadow(shadow);

        assert!(manager.shadow_component.is_none()); // Update should be rejected
    }
}
