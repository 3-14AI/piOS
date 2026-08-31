//! Agent-to-Agent (A2A) IPC Protocol
//! Defines standard messaging structure for AI agents to communicate.

use super::priority::AgentPriority;

/// Types of messages an agent can send to another agent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageType {
    /// A query for information or data.
    Query,
    /// A command to perform an action.
    Command,
    /// A response to a previous query or command.
    Response,
    /// A broadcast announcement of state change or availability.
    Announcement,
    /// An error reporting message.
    Error,
}

/// A standard A2A message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct A2AMessage {
    /// Sender agent ID.
    pub sender_id: u32,
    /// Receiver agent ID (or 0 for broadcast).
    pub receiver_id: u32,
    /// The type of message.
    pub msg_type: MessageType,
    /// Priority of the message, used by the scheduler.
    pub priority: AgentPriority,
    /// Payload length.
    pub payload_len: usize,
    /// The actual data payload.
    // In a real no_std environment we might use a fixed-size buffer or an allocated Vec if alloc is available.
    // We will simulate it with a simple slice or array for now.
    pub payload: [u8; 256],
}

impl A2AMessage {
    pub fn new(
        sender_id: u32,
        receiver_id: u32,
        msg_type: MessageType,
        priority: AgentPriority,
        payload_data: &[u8],
    ) -> Self {
        let mut payload = [0u8; 256];
        let payload_len = payload_data.len().min(256);
        payload[..payload_len].copy_from_slice(&payload_data[..payload_len]);

        Self {
            sender_id,
            receiver_id,
            msg_type,
            priority,
            payload_len,
            payload,
        }
    }

    pub fn serialize(&self) -> alloc::vec::Vec<u8> {
        let mut buf = alloc::vec::Vec::new();
        buf.extend_from_slice(&self.sender_id.to_le_bytes());
        buf.extend_from_slice(&self.receiver_id.to_le_bytes());

        let msg_type_byte = match self.msg_type {
            MessageType::Query => 0,
            MessageType::Command => 1,
            MessageType::Response => 2,
            MessageType::Announcement => 3,
            MessageType::Error => 4,
        };
        buf.push(msg_type_byte);

        let priority_byte = match self.priority {
            AgentPriority::Background => 0,
            AgentPriority::Normal => 1,
            AgentPriority::High => 2,
            AgentPriority::Critical => 3,
        };
        buf.push(priority_byte);

        buf.extend_from_slice(&(self.payload_len as u32).to_le_bytes());
        buf.extend_from_slice(&self.payload);
        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let data = b"Hello Agent 2";
        let msg = A2AMessage::new(1, 2, MessageType::Command, AgentPriority::Normal, data);

        assert_eq!(msg.sender_id, 1);
        assert_eq!(msg.receiver_id, 2);
        assert_eq!(msg.msg_type, MessageType::Command);
        assert_eq!(msg.priority, AgentPriority::Normal);
        assert_eq!(msg.payload_len, data.len());
        assert_eq!(&msg.payload[..msg.payload_len], data);
    }
}
