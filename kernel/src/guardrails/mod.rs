#![allow(unused_imports)]
extern crate alloc;
use alloc::vec::Vec;

#[cfg(feature = "verus")]
use vstd::prelude::*;

#[cfg(feature = "verus")]
verus! {
    /// Agent Identifier
    #[derive(Copy, Clone)]
    pub struct AgentId(pub u64);

    /// Resource Identifier
    #[derive(Copy, Clone)]
    pub struct ResourceId(pub u64);

    /// Abstract capability policy
    #[derive(Copy, Clone)]
    pub enum AclPolicy {
        Allow,
        Deny,
    }

    impl AclPolicy {
        pub open spec fn eq(self, other: AclPolicy) -> bool {
            match self {
                AclPolicy::Allow => match other { AclPolicy::Allow => true, _ => false },
                AclPolicy::Deny => match other { AclPolicy::Deny => true, _ => false },
            }
        }

        pub fn is_eq(&self, other: &AclPolicy) -> (res: bool)
            ensures res == self.eq(*other)
        {
            match self {
                AclPolicy::Allow => match other { AclPolicy::Allow => true, _ => false },
                AclPolicy::Deny => match other { AclPolicy::Deny => true, _ => false },
            }
        }
    }

    /// Access Control List entry
    #[derive(Copy, Clone)]
    pub struct AclEntry {
        pub agent: AgentId,
        pub resource: ResourceId,
        pub policy: AclPolicy,
    }

    /// Guardrail System
    pub struct Guardrail {
        pub entries: Vec<AclEntry>,
    }

    impl Guardrail {
        pub fn new() -> (g: Self)
            ensures g.entries.len() == 0
        {
            Guardrail {
                entries: Vec::new(),
            }
        }

        pub fn add_rule(&mut self, agent: AgentId, resource: ResourceId, policy: AclPolicy)
            ensures
                self.entries.len() == old(self).entries.len() + 1,
                self.entries[self.entries.len() - 1].agent.0 == agent.0,
                self.entries[self.entries.len() - 1].resource.0 == resource.0,
                self.entries[self.entries.len() - 1].policy.eq(policy)
        {
            let entry = AclEntry { agent, resource, policy };
            self.entries.push(entry);
        }

        pub fn check_action(&self, agent: AgentId, resource: ResourceId) -> (allowed: bool)
            ensures
                allowed ==> exists|i: int| #![auto] 0 <= i && i < self.entries.len() &&
                    self.entries[i].agent.0 == agent.0 &&
                    self.entries[i].resource.0 == resource.0 &&
                    self.entries[i].policy.eq(AclPolicy::Allow),
                !allowed ==> forall|i: int| #![auto] 0 <= i && i < self.entries.len() ==>
                    self.entries[i].agent.0 != agent.0 ||
                    self.entries[i].resource.0 != resource.0 ||
                    !self.entries[i].policy.eq(AclPolicy::Allow)
        {
            let mut i = 0;
            let mut found_allow = false;

            while i < self.entries.len()
                invariant
                    0 <= i && i <= self.entries.len(),
                    found_allow ==> exists|j: int| #![auto] 0 <= j && j < i &&
                        self.entries[j].agent.0 == agent.0 &&
                        self.entries[j].resource.0 == resource.0 &&
                        self.entries[j].policy.eq(AclPolicy::Allow),
                    !found_allow ==> forall|j: int| #![auto] 0 <= j && j < i ==>
                        self.entries[j].agent.0 != agent.0 ||
                        self.entries[j].resource.0 != resource.0 ||
                        !self.entries[j].policy.eq(AclPolicy::Allow)
                decreases self.entries.len() - i
            {
                if self.entries[i].agent.0 == agent.0 && self.entries[i].resource.0 == resource.0 {
                    if self.entries[i].policy.is_eq(&AclPolicy::Allow) {
                        found_allow = true;
                    }
                }
                i = i + 1;
            }
            found_allow
        }
    }
}

#[cfg(not(feature = "verus"))]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct AgentId(pub u64);

#[cfg(not(feature = "verus"))]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct ResourceId(pub u64);

#[cfg(not(feature = "verus"))]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum AclPolicy {
    Allow,
    Deny,
}

#[cfg(not(feature = "verus"))]
#[derive(Copy, Clone)]
pub struct AclEntry {
    pub agent: AgentId,
    pub resource: ResourceId,
    pub policy: AclPolicy,
}

#[cfg(not(feature = "verus"))]
pub struct Guardrail {
    pub entries: Vec<AclEntry>,
}

#[cfg(not(feature = "verus"))]
impl Default for Guardrail {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "verus"))]
impl Guardrail {
    pub fn new() -> Self {
        Guardrail {
            entries: Vec::new(),
        }
    }

    pub fn add_rule(&mut self, agent: AgentId, resource: ResourceId, policy: AclPolicy) {
        self.entries.push(AclEntry { agent, resource, policy });
    }

    pub fn check_action(&self, agent: AgentId, resource: ResourceId) -> bool {
        for entry in &self.entries {
            if entry.agent.0 == agent.0 && entry.resource.0 == resource.0 {
                if entry.policy == AclPolicy::Allow {
                    return true;
                }
            }
        }
        false
    }
}

#[cfg(not(feature = "verus"))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guardrail_allow_deny() {
        let mut guardrail = Guardrail::new();

        let agent1 = AgentId(1);
        let agent2 = AgentId(2);
        let resource1 = ResourceId(10);
        let resource2 = ResourceId(20);

        guardrail.add_rule(agent1, resource1, AclPolicy::Allow);
        guardrail.add_rule(agent1, resource2, AclPolicy::Deny);
        guardrail.add_rule(agent2, resource1, AclPolicy::Deny);

        // Explicit allow
        assert!(guardrail.check_action(agent1, resource1));

        // Explicit deny
        assert!(!guardrail.check_action(agent1, resource2));
        assert!(!guardrail.check_action(agent2, resource1));

        // Implicit deny (no rule)
        assert!(!guardrail.check_action(agent2, resource2));
    }
}
