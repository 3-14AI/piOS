use vstd::prelude::*;

verus! {
    #[derive(Clone, Copy)]
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

        pub fn update_shadow(&mut self, new_shadow: Component)
            requires
                new_shadow.version > old(self).primary_component.version,
            ensures
                self.shadow_component == Some(new_shadow),
                self.primary_component == old(self).primary_component,
        {
            self.shadow_component = Some(new_shadow);
        }

        pub fn execute_and_compare(&self, input: u32) -> (result: bool)
            requires
                self.shadow_component matches Some(_),
                self.primary_component.data <= u32::MAX - input,
                self.shadow_component.unwrap().data <= u32::MAX - input,
            ensures
                result == ((self.primary_component.data + input) == (self.shadow_component.unwrap().data + input)),
        {
            let primary_result = self.primary_component.data + input;
            let shadow_result = self.shadow_component.unwrap().data + input;
            primary_result == shadow_result
        }

        pub fn promote_shadow(&mut self)
            requires
                old(self).shadow_component matches Some(_),
            ensures
                self.primary_component == old(self).shadow_component.unwrap(),
                self.shadow_component matches None,
        {
            self.primary_component = self.shadow_component.unwrap();
            self.shadow_component = None;
        }
    }
}
