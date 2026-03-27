use vstd::prelude::*;

verus! {
    pub struct ComponentState {
        pub version: u32,
        pub data: u32,
    }

    /// Migrates the state of a component to a new version.
    /// Ensures that the new state has the target version,
    /// and that the underlying data is preserved.
    pub fn migrate_state(old_state: ComponentState, new_version: u32) -> (new_state: ComponentState)
        requires
            new_version > old_state.version,
        ensures
            new_state.version == new_version,
            new_state.data == old_state.data,
    {
        ComponentState {
            version: new_version,
            data: old_state.data,
        }
    }
}
