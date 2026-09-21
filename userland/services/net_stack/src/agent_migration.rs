extern crate alloc;
use crate::WasmNetStack;
use alloc::vec::Vec;
use smoltcp::iface::SocketHandle;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationMessage {
    InitiateMigration {
        agent_id: u32,
        target_node_id: u32,
        memory_state: Vec<u8>,
    },
    MigrationAck {
        agent_id: u32,
        node_id: u32,
        success: bool,
    },
}

impl MigrationMessage {
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        match self {
            Self::InitiateMigration {
                agent_id,
                target_node_id,
                memory_state,
            } => {
                buf.push(0);
                buf.extend_from_slice(&agent_id.to_le_bytes());
                buf.extend_from_slice(&target_node_id.to_le_bytes());
                buf.extend_from_slice(&(memory_state.len() as u32).to_le_bytes());
                buf.extend_from_slice(memory_state);
            }
            Self::MigrationAck {
                agent_id,
                node_id,
                success,
            } => {
                buf.push(1);
                buf.extend_from_slice(&agent_id.to_le_bytes());
                buf.extend_from_slice(&node_id.to_le_bytes());
                buf.push(if *success { 1 } else { 0 });
            }
        }
        buf
    }

    pub fn deserialize(data: &[u8]) -> Option<Self> {
        if data.is_empty() {
            return None;
        }
        match data[0] {
            0 => {
                if data.len() < 13 {
                    return None;
                }
                let agent_id = u32::from_le_bytes(data[1..5].try_into().unwrap());
                let target_node_id = u32::from_le_bytes(data[5..9].try_into().unwrap());
                let state_len = u32::from_le_bytes(data[9..13].try_into().unwrap()) as usize;
                let total_len = 13usize.checked_add(state_len)?;
                if data.len() < total_len {
                    return None;
                }
                let memory_state = data[13..total_len].to_vec();
                Some(Self::InitiateMigration {
                    agent_id,
                    target_node_id,
                    memory_state,
                })
            }
            1 => {
                if data.len() < 10 {
                    return None;
                }
                let agent_id = u32::from_le_bytes(data[1..5].try_into().unwrap());
                let node_id = u32::from_le_bytes(data[5..9].try_into().unwrap());
                let success = data[9] != 0;
                Some(Self::MigrationAck {
                    agent_id,
                    node_id,
                    success,
                })
            }
            _ => None,
        }
    }
}

pub struct AgentMigrationManager {
    pub node_id: u32,
    socket_handle: SocketHandle,
    port: u16,
    pub active_migrations: Vec<u32>,
    pub hosted_agents: Vec<u32>,
}

impl AgentMigrationManager {
    pub fn new(stack: &mut WasmNetStack, node_id: u32, port: u16) -> Self {
        let socket_handle = stack.add_udp_broadcast_socket(port);
        Self {
            node_id,
            socket_handle,
            port,
            active_migrations: Vec::new(),
            hosted_agents: Vec::new(),
        }
    }

    pub fn migrate_agent(
        &mut self,
        stack: &mut WasmNetStack,
        agent_id: u32,
        target_node_id: u32,
        memory_state: Vec<u8>,
    ) -> Result<(), &'static str> {
        if !self.hosted_agents.contains(&agent_id) {
            return Err("Agent not hosted on this node");
        }
        self.active_migrations.push(agent_id);
        let msg = MigrationMessage::InitiateMigration {
            agent_id,
            target_node_id,
            memory_state,
        };
        stack.send_udp_broadcast(self.socket_handle, self.port, &msg.serialize());
        Ok(())
    }

    pub fn process_packet(&mut self, stack: &mut WasmNetStack, data: &[u8]) {
        if let Some(msg) = MigrationMessage::deserialize(data) {
            match msg {
                MigrationMessage::InitiateMigration {
                    agent_id,
                    target_node_id,
                    memory_state: _,
                } => {
                    if target_node_id == self.node_id {
                        // Accept migration
                        if !self.hosted_agents.contains(&agent_id) {
                            self.hosted_agents.push(agent_id);
                        }
                        let reply = MigrationMessage::MigrationAck {
                            agent_id,
                            node_id: self.node_id,
                            success: true,
                        };
                        stack.send_udp_broadcast(self.socket_handle, self.port, &reply.serialize());
                    }
                }
                MigrationMessage::MigrationAck {
                    agent_id,
                    node_id: _,
                    success,
                } => {
                    if success {
                        if let Some(pos) =
                            self.active_migrations.iter().position(|&x| x == agent_id)
                        {
                            self.active_migrations.remove(pos);
                            if let Some(hosted_pos) =
                                self.hosted_agents.iter().position(|&x| x == agent_id)
                            {
                                self.hosted_agents.remove(hosted_pos);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_message_serialization() {
        let msg = MigrationMessage::InitiateMigration {
            agent_id: 42,
            target_node_id: 2,
            memory_state: alloc::vec![1, 2, 3],
        };
        let data = msg.serialize();
        let decoded = MigrationMessage::deserialize(&data).unwrap();
        assert_eq!(msg, decoded);

        let ack = MigrationMessage::MigrationAck {
            agent_id: 42,
            node_id: 2,
            success: true,
        };
        let ack_data = ack.serialize();
        let ack_decoded = MigrationMessage::deserialize(&ack_data).unwrap();
        assert_eq!(ack, ack_decoded);
    }

    #[test]
    fn test_agent_migration_flow() {
        let mut stack = WasmNetStack::new();
        let mut manager1 = AgentMigrationManager::new(&mut stack, 1, 9998);
        manager1.hosted_agents.push(42);

        let mut manager2 = AgentMigrationManager::new(&mut stack, 2, 9998);

        // Initiate
        assert!(manager1
            .migrate_agent(&mut stack, 42, 2, alloc::vec![1, 2, 3])
            .is_ok());
        assert_eq!(manager1.active_migrations, alloc::vec![42]);

        // Manager 2 receives packet
        let init_msg = MigrationMessage::InitiateMigration {
            agent_id: 42,
            target_node_id: 2,
            memory_state: alloc::vec![1, 2, 3],
        };
        manager2.process_packet(&mut stack, &init_msg.serialize());
        assert_eq!(manager2.hosted_agents, alloc::vec![42]);

        // Manager 1 receives ACK
        let ack_msg = MigrationMessage::MigrationAck {
            agent_id: 42,
            node_id: 2,
            success: true,
        };
        manager1.process_packet(&mut stack, &ack_msg.serialize());
        assert!(manager1.active_migrations.is_empty());
        assert!(manager1.hosted_agents.is_empty());
    }
}
