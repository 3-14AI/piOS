extern crate alloc;
use crate::WasmNetStack;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use llm_inference::LlmInference;
use smoltcp::iface::SocketHandle;

// Stub for native testing; this will be resolved properly in WASM context by Cranelift Linker
#[cfg(not(test))]
extern "C" {
    pub fn wasi_ephemeral_compiler(code_ptr: *const u8, code_len: i32) -> i32;
}

/// # Safety
/// This is a mock function for testing that simulates compiling wasm ephemeral code.
/// It does not perform actual unsafe operations but requires an unsafe block for FFI compatibility.
#[cfg(test)]
pub unsafe fn wasi_ephemeral_compiler(_code_ptr: *const u8, _code_len: i32) -> i32 {
    0 // mock SUCCESS
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasmExecutionState {
    pub memory: Vec<u8>,
    pub stack: Vec<u8>,
    pub descriptors: Vec<u32>,
    pub wasm_source: Vec<u8>,
}

impl WasmExecutionState {
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&(self.memory.len() as u32).to_le_bytes());
        buf.extend_from_slice(&self.memory);

        buf.extend_from_slice(&(self.stack.len() as u32).to_le_bytes());
        buf.extend_from_slice(&self.stack);

        buf.extend_from_slice(&(self.descriptors.len() as u32).to_le_bytes());
        for desc in &self.descriptors {
            buf.extend_from_slice(&desc.to_le_bytes());
        }

        buf.extend_from_slice(&(self.wasm_source.len() as u32).to_le_bytes());
        buf.extend_from_slice(&self.wasm_source);

        buf
    }

    pub fn deserialize(data: &[u8]) -> Option<(Self, usize)> {
        let mut offset = 0;

        if data.len() < offset + 4 {
            return None;
        }
        let mem_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        if data.len() < offset + mem_len {
            return None;
        }
        let memory = data[offset..offset + mem_len].to_vec();
        offset += mem_len;

        if data.len() < offset + 4 {
            return None;
        }
        let stack_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        if data.len() < offset + stack_len {
            return None;
        }
        let stack = data[offset..offset + stack_len].to_vec();
        offset += stack_len;

        if data.len() < offset + 4 {
            return None;
        }
        let desc_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        let mut descriptors = Vec::with_capacity(desc_len);
        for _ in 0..desc_len {
            if data.len() < offset + 4 {
                return None;
            }
            descriptors.push(u32::from_le_bytes(
                data[offset..offset + 4].try_into().unwrap(),
            ));
            offset += 4;
        }

        if data.len() < offset + 4 {
            return None;
        }
        let source_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        if data.len() < offset + source_len {
            return None;
        }
        let wasm_source = data[offset..offset + source_len].to_vec();
        offset += source_len;

        Some((
            Self {
                memory,
                stack,
                descriptors,
                wasm_source,
            },
            offset,
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationMessage {
    InitiateMigration {
        agent_id: u32,
        target_node_id: u32,
        execution_state: WasmExecutionState,
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
                execution_state,
            } => {
                buf.push(0);
                buf.extend_from_slice(&agent_id.to_le_bytes());
                buf.extend_from_slice(&target_node_id.to_le_bytes());
                let state_bytes = execution_state.serialize();
                buf.extend_from_slice(&(state_bytes.len() as u32).to_le_bytes());
                buf.extend_from_slice(&state_bytes);
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
                let (execution_state, _) = WasmExecutionState::deserialize(&data[13..total_len])?;
                Some(Self::InitiateMigration {
                    agent_id,
                    target_node_id,
                    execution_state,
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
    llm: LlmInference,
    pub rollback_states: alloc::collections::BTreeMap<u32, WasmExecutionState>,
}

impl AgentMigrationManager {
    pub fn new(stack: &mut WasmNetStack, node_id: u32, port: u16) -> Self {
        let socket_handle = stack.add_udp_broadcast_socket(port);
        let mut llm = LlmInference::new();
        let _ = llm.load_model("migration_model");
        Self {
            node_id,
            socket_handle,
            port,
            active_migrations: Vec::new(),
            hosted_agents: Vec::new(),
            llm,
            rollback_states: alloc::collections::BTreeMap::new(),
        }
    }

    pub fn migrate_agent(
        &mut self,
        stack: &mut WasmNetStack,
        agent_id: u32,
        target_node_id: u32,
        execution_state: WasmExecutionState,
    ) -> Result<(), &'static str> {
        if !self.hosted_agents.contains(&agent_id) {
            return Err("Agent not hosted on this node");
        }
        self.active_migrations.push(agent_id);

        // Save state for rollback in case of failure
        self.rollback_states
            .insert(agent_id, execution_state.clone());

        let msg = MigrationMessage::InitiateMigration {
            agent_id,
            target_node_id,
            execution_state,
        };
        stack.send_udp_broadcast(self.socket_handle, self.port, &msg.serialize());
        Ok(())
    }

    pub fn monitor_load_and_migrate(
        &mut self,
        stack: &mut WasmNetStack,
        cpu_load_percent: u8,
        memory_load_percent: u8,
        thermal_profile: u8,
        available_nodes: &[u32],
    ) -> Result<(), &'static str> {
        if available_nodes.is_empty() {
            return Ok(()); // Nothing to do
        }

        // Convert node slice to a comma separated string to include in the prompt
        let nodes_str = available_nodes
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<String>>()
            .join(", ");

        let prompt = alloc::format!("Analyze system health: CPU {}%, RAM {}%, Thermal {}C. Available target nodes: [{}]. Should we migrate an agent to reduce load? If yes, respond with 'MIGRATE TO NODE X' where X is the optimal node ID.", cpu_load_percent, memory_load_percent, thermal_profile, nodes_str);

        // Attempt to infer from LLM. If LLM fails, we fallback to our heuristic.
        let target_node_id = if let Ok(response) = self.llm.infer(&prompt) {
            // Check if the LLM suggested migration
            if response.contains("MIGRATE TO NODE") {
                // Extremely simple parsing: Find "NODE X", parse X
                let parts: Vec<&str> = response.split("NODE ").collect();
                if parts.len() > 1 {
                    let maybe_id = parts[1]
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .trim_matches(|c: char| !c.is_numeric());
                    if let Ok(id) = maybe_id.parse::<u32>() {
                        if available_nodes.contains(&id) {
                            id
                        } else {
                            available_nodes[0]
                        }
                    } else {
                        available_nodes[0]
                    }
                } else {
                    available_nodes[0]
                }
            } else {
                return Ok(()); // No migration needed according to LLM
            }
        } else {
            // Fallback High load threshold
            if cpu_load_percent > 85 || memory_load_percent > 85 || thermal_profile > 90 {
                available_nodes[0]
            } else {
                return Ok(()); // No migration needed
            }
        };

        // We migrate the first hosted agent to reduce load
        if let Some(&agent_to_migrate) = self.hosted_agents.first() {
            // Extracting live state is mocked until deep OS integrations are complete.
            let actual_state = WasmExecutionState {
                memory: alloc::vec![],
                stack: alloc::vec![],
                descriptors: alloc::vec![],
                wasm_source: alloc::vec![],
            };
            return self.migrate_agent(stack, agent_to_migrate, target_node_id, actual_state);
        }

        Ok(())
    }

    pub fn process_packet(&mut self, stack: &mut WasmNetStack, data: &[u8]) {
        if let Some(msg) = MigrationMessage::deserialize(data) {
            match msg {
                MigrationMessage::InitiateMigration {
                    agent_id,
                    target_node_id,
                    execution_state,
                } => {
                    if target_node_id == self.node_id {
                        // Accept migration
                        if !self.hosted_agents.contains(&agent_id) {
                            self.hosted_agents.push(agent_id);
                        }

                        // We attempt to dynamically re-compile/instantiate the received WASM
                        let compile_success = if execution_state.wasm_source.is_empty() {
                            true // mock success if nothing to compile
                        } else {
                            let res = unsafe {
                                wasi_ephemeral_compiler(
                                    execution_state.wasm_source.as_ptr(),
                                    execution_state.wasm_source.len() as i32,
                                )
                            };
                            res == 0 // WASI_ERRNO_SUCCESS typically 0
                        };

                        let reply = MigrationMessage::MigrationAck {
                            agent_id,
                            node_id: self.node_id,
                            success: compile_success,
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
                        // Migration succeeded
                        if let Some(pos) =
                            self.active_migrations.iter().position(|&x| x == agent_id)
                        {
                            self.active_migrations.remove(pos);
                            if let Some(hosted_pos) =
                                self.hosted_agents.iter().position(|&x| x == agent_id)
                            {
                                self.hosted_agents.remove(hosted_pos);
                            }
                            // Clean up rollback state since we succeeded
                            self.rollback_states.remove(&agent_id);
                        }
                    } else {
                        // Migration failed - Rollback
                        if let Some(pos) =
                            self.active_migrations.iter().position(|&x| x == agent_id)
                        {
                            self.active_migrations.remove(pos);
                        }

                        // Agent remains hosted on this node. We can optionally restore state here from `rollback_states`
                        if let Some(_state) = self.rollback_states.remove(&agent_id) {
                            // restore_live_state(agent_id, state);
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
        let execution_state = WasmExecutionState {
            memory: alloc::vec![1, 2, 3],
            stack: alloc::vec![4, 5, 6],
            descriptors: alloc::vec![0, 1, 2],
            wasm_source: alloc::vec![7, 8, 9],
        };
        let msg = MigrationMessage::InitiateMigration {
            agent_id: 42,
            target_node_id: 2,
            execution_state,
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

        let execution_state = WasmExecutionState {
            memory: alloc::vec![1, 2, 3],
            stack: alloc::vec![4, 5, 6],
            descriptors: alloc::vec![0, 1, 2],
            wasm_source: alloc::vec![7, 8, 9],
        };

        // Initiate
        assert!(manager1
            .migrate_agent(&mut stack, 42, 2, execution_state.clone())
            .is_ok());
        assert_eq!(manager1.active_migrations, alloc::vec![42]);

        // Manager 2 receives packet
        let init_msg = MigrationMessage::InitiateMigration {
            agent_id: 42,
            target_node_id: 2,
            execution_state,
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

    #[test]
    fn test_monitor_load_and_migrate_llm() {
        let mut stack = WasmNetStack::new();
        let mut manager1 = AgentMigrationManager::new(&mut stack, 1, 9998);
        manager1.hosted_agents.push(42);

        let available_nodes = alloc::vec![2, 3];
        let res = manager1.monitor_load_and_migrate(&mut stack, 90, 90, 95, &available_nodes);
        assert!(res.is_ok());
    }

    #[test]
    fn test_migration_rollback_on_failure() {
        let mut stack = WasmNetStack::new();
        let mut manager1 = AgentMigrationManager::new(&mut stack, 1, 9998);
        manager1.hosted_agents.push(42);

        let execution_state = WasmExecutionState {
            memory: alloc::vec![],
            stack: alloc::vec![],
            descriptors: alloc::vec![],
            wasm_source: alloc::vec![],
        };

        manager1
            .migrate_agent(&mut stack, 42, 2, execution_state.clone())
            .unwrap();
        assert!(manager1.active_migrations.contains(&42));
        assert!(manager1.rollback_states.contains_key(&42));

        // Simulate failure ACK
        let ack_msg = MigrationMessage::MigrationAck {
            agent_id: 42,
            node_id: 2,
            success: false, // FAILURE
        };
        manager1.process_packet(&mut stack, &ack_msg.serialize());

        // Agent should no longer be in active_migrations, but still in hosted_agents
        assert!(!manager1.active_migrations.contains(&42));
        assert!(manager1.hosted_agents.contains(&42));
        // Rollback state should be cleared
        assert!(!manager1.rollback_states.contains_key(&42));
    }
}
