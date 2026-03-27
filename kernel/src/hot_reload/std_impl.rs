pub struct ComponentState {
    pub version: u32,
    pub data: u32,
}

pub fn migrate_state(old_state: ComponentState, new_version: u32) -> ComponentState {
    ComponentState {
        version: new_version,
        data: old_state.data,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrate_state() {
        let old_state = ComponentState {
            version: 1,
            data: 42,
        };
        let new_state = migrate_state(old_state, 2);
        assert_eq!(new_state.version, 2);
        assert_eq!(new_state.data, 42);
    }
}
