//! Agent Priority Definitions
//! Specifies priority levels for A2A communication, used by the OS scheduler
//! and IPC mechanisms to order agent execution and message delivery.

/// Defines the importance of a task or an agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AgentPriority {
    /// Lowest priority, background tasks like logging and telemetry.
    Background = 0,
    /// Normal priority, standard user-level applications.
    Normal = 1,
    /// High priority, interactive tasks requiring quick response.
    High = 2,
    /// Critical priority, system-level tasks that must not be delayed (e.g., security monitors).
    Critical = 3,
}

impl AgentPriority {
    /// Determines if this priority level can preempt another.
    pub fn can_preempt(&self, other: &Self) -> bool {
        self > other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_ordering() {
        assert!(AgentPriority::Critical > AgentPriority::High);
        assert!(AgentPriority::High > AgentPriority::Normal);
        assert!(AgentPriority::Normal > AgentPriority::Background);
    }

    #[test]
    fn test_preemption() {
        assert!(AgentPriority::Critical.can_preempt(&AgentPriority::Normal));
        assert!(!AgentPriority::Background.can_preempt(&AgentPriority::High));
        assert!(!AgentPriority::Normal.can_preempt(&AgentPriority::Normal));
    }
}
