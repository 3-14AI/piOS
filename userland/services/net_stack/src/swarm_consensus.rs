extern crate alloc;
use crate::WasmNetStack;
use smoltcp::iface::SocketHandle;
use alloc::vec::Vec;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SwarmState {
    Follower,
    Candidate,
    Leader,
    CatchingUp,
    Failed,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ConsensusMessage {
    RequestVote {
        term: u64,
        candidate_id: u32,
    },
    Vote {
        term: u64,
        voter_id: u32,
        vote_granted: bool,
    },
    ProposeTask {
        term: u64,
        leader_id: u32,
        task_id: u32,
        task_payload: Vec<u8>,
    },
    PrepareTask {
        term: u64,
        voter_id: u32,
        task_id: u32,
    },
    CommitTask {
        term: u64,
        voter_id: u32,
        task_id: u32,
    },
}

impl ConsensusMessage {
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        match self {
            Self::RequestVote { term, candidate_id } => {
                buf.push(0);
                buf.extend_from_slice(&term.to_le_bytes());
                buf.extend_from_slice(&candidate_id.to_le_bytes());
            }
            Self::Vote {
                term,
                voter_id,
                vote_granted,
            } => {
                buf.push(1);
                buf.extend_from_slice(&term.to_le_bytes());
                buf.extend_from_slice(&voter_id.to_le_bytes());
                buf.push(if *vote_granted { 1 } else { 0 });
            }
            Self::ProposeTask {
                term,
                leader_id,
                task_id,
                task_payload,
            } => {
                buf.push(2);
                buf.extend_from_slice(&term.to_le_bytes());
                buf.extend_from_slice(&leader_id.to_le_bytes());
                buf.extend_from_slice(&task_id.to_le_bytes());
                buf.extend_from_slice(&(task_payload.len() as u32).to_le_bytes());
                buf.extend_from_slice(task_payload);
            }
            Self::PrepareTask { term, voter_id, task_id } => {
                buf.push(3);
                buf.extend_from_slice(&term.to_le_bytes());
                buf.extend_from_slice(&voter_id.to_le_bytes());
                buf.extend_from_slice(&task_id.to_le_bytes());
            }
            Self::CommitTask { term, voter_id, task_id } => {
                buf.push(4);
                buf.extend_from_slice(&term.to_le_bytes());
                buf.extend_from_slice(&voter_id.to_le_bytes());
                buf.extend_from_slice(&task_id.to_le_bytes());
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
                let term = u64::from_le_bytes(data[1..9].try_into().unwrap());
                let candidate_id = u32::from_le_bytes(data[9..13].try_into().unwrap());
                Some(Self::RequestVote { term, candidate_id })
            }
            1 => {
                if data.len() < 14 {
                    return None;
                }
                let term = u64::from_le_bytes(data[1..9].try_into().unwrap());
                let voter_id = u32::from_le_bytes(data[9..13].try_into().unwrap());
                let vote_granted = data[13] != 0;
                Some(Self::Vote {
                    term,
                    voter_id,
                    vote_granted,
                })
            }
            2 => {
                if data.len() < 21 {
                    return None;
                }
                let term = u64::from_le_bytes(data[1..9].try_into().unwrap());
                let leader_id = u32::from_le_bytes(data[9..13].try_into().unwrap());
                let task_id = u32::from_le_bytes(data[13..17].try_into().unwrap());
                let payload_len = u32::from_le_bytes(data[17..21].try_into().unwrap()) as usize;
                let total_len = 21usize.checked_add(payload_len)?;
                if data.len() < total_len {
                    return None;
                }
                let task_payload = data[21..total_len].to_vec();
                Some(Self::ProposeTask {
                    term,
                    leader_id,
                    task_id,
                    task_payload,
                })
            }
            3 => {
                if data.len() < 17 {
                    return None;
                }
                let term = u64::from_le_bytes(data[1..9].try_into().unwrap());
                let voter_id = u32::from_le_bytes(data[9..13].try_into().unwrap());
                let task_id = u32::from_le_bytes(data[13..17].try_into().unwrap());
                Some(Self::PrepareTask { term, voter_id, task_id })
            }
            4 => {
                if data.len() < 17 {
                    return None;
                }
                let term = u64::from_le_bytes(data[1..9].try_into().unwrap());
                let voter_id = u32::from_le_bytes(data[9..13].try_into().unwrap());
                let task_id = u32::from_le_bytes(data[13..17].try_into().unwrap());
                Some(Self::CommitTask { term, voter_id, task_id })
            }
            _ => None,
        }
    }
}

pub struct SwarmConsensus {
    pub node_id: u32,
    pub state: SwarmState,
    pub current_term: u64,
    pub voted_for: Option<u32>,
    pub votes_received: Vec<u32>,
    pub task_prepares: Vec<u32>,
    pub task_commits: Vec<u32>,
    socket_handle: SocketHandle,
    port: u16,
}

impl SwarmConsensus {
    pub fn new(stack: &mut WasmNetStack, node_id: u32, port: u16) -> Self {
        let socket_handle = stack.add_udp_broadcast_socket(port);
        Self {
            node_id,
            state: SwarmState::Follower,
            current_term: 0,
            voted_for: None,
            votes_received: alloc::vec::Vec::new(),
            task_prepares: alloc::vec::Vec::new(),
            task_commits: alloc::vec::Vec::new(),
            socket_handle,
            port,
        }
    }

    pub fn start_election(&mut self, stack: &mut WasmNetStack) {
        if self.state == SwarmState::Failed {
            return;
        }
        self.state = SwarmState::Candidate;
        self.current_term += 1;
        self.voted_for = Some(self.node_id);
        self.votes_received = alloc::vec![self.node_id]; // Vote for self

        let msg = ConsensusMessage::RequestVote {
            term: self.current_term,
            candidate_id: self.node_id,
        };
        let data = msg.serialize();
        stack.send_udp_broadcast(self.socket_handle, self.port, &data);
    }

    pub fn propose_task(
        &mut self,
        stack: &mut WasmNetStack,
        task_id: u32,
        payload: Vec<u8>,
    ) -> Result<(), &'static str> {
        if self.state != SwarmState::Leader {
            return Err("Only leader can propose tasks");
        }
        self.task_prepares.clear();
        self.task_commits.clear();

        self.task_prepares.push(self.node_id);
        self.task_commits.push(self.node_id);

        let msg = ConsensusMessage::ProposeTask {
            term: self.current_term,
            leader_id: self.node_id,
            task_id,
            task_payload: payload,
        };
        let data = msg.serialize();
        stack.send_udp_broadcast(self.socket_handle, self.port, &data);
        Ok(())
    }

    pub fn process_packet(&mut self, stack: &mut WasmNetStack, data: &[u8], cluster_size: usize) {
        if self.state == SwarmState::Failed {
            return;
        }
        let bft_threshold = (cluster_size * 2) / 3;

        if let Some(msg) = ConsensusMessage::deserialize(data) {
            match msg {
                ConsensusMessage::RequestVote { term, candidate_id } => {
                    if term > self.current_term {
                        self.current_term = term;
                        self.state = SwarmState::Follower;
                        self.voted_for = None;
                    }

                    let vote_granted = if term == self.current_term
                        && (self.voted_for.is_none() || self.voted_for == Some(candidate_id))
                    {
                        self.voted_for = Some(candidate_id);
                        true
                    } else {
                        false
                    };

                    let reply = ConsensusMessage::Vote {
                        term: self.current_term,
                        voter_id: self.node_id,
                        vote_granted,
                    };
                    stack.send_udp_broadcast(self.socket_handle, self.port, &reply.serialize());
                }
                ConsensusMessage::Vote {
                    term,
                    voter_id,
                    vote_granted,
                } => {
                    if self.state == SwarmState::Candidate
                        && term == self.current_term
                        && vote_granted
                        && !self.votes_received.contains(&voter_id)
                    {
                        self.votes_received.push(voter_id);
                        if self.votes_received.len() > bft_threshold {
                            self.state = SwarmState::Leader;
                        }
                    }
                }
                ConsensusMessage::ProposeTask {
                    term,
                    leader_id: _,
                    task_id,
                    task_payload: _,
                } => {
                    if term >= self.current_term {
                        self.current_term = term;
                        self.state = SwarmState::Follower;

                        // Clear existing prepares/commits on a new proposal
                        self.task_prepares.clear();
                        self.task_commits.clear();

                        // Self-vote for the new proposal
                        self.task_prepares.push(self.node_id);
                        self.task_commits.push(self.node_id);

                        // Broadcast PrepareTask (PBFT Prepare phase)
                        let reply = ConsensusMessage::PrepareTask {
                            term: self.current_term,
                            voter_id: self.node_id,
                            task_id,
                        };
                        stack.send_udp_broadcast(self.socket_handle, self.port, &reply.serialize());
                    }
                }
                ConsensusMessage::PrepareTask {
                    term,
                    voter_id,
                    task_id,
                } => {
                    if term >= self.current_term {
                        if !self.task_prepares.contains(&voter_id) {
                            self.task_prepares.push(voter_id);
                        }
                        if self.task_prepares.len() > bft_threshold {
                            // Broadcast CommitTask (PBFT Commit phase)
                            let reply = ConsensusMessage::CommitTask {
                                term: self.current_term,
                                voter_id: self.node_id,
                                task_id,
                            };
                            stack.send_udp_broadcast(self.socket_handle, self.port, &reply.serialize());
                        }
                    }
                }
                ConsensusMessage::CommitTask {
                    term,
                    voter_id,
                    task_id: _,
                } => {
                    if term >= self.current_term {
                        if !self.task_commits.contains(&voter_id) {
                            self.task_commits.push(voter_id);
                        }
                        if self.task_commits.len() > bft_threshold {
                            // Task fully committed in a BFT manner
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
    fn test_initial_state() {
        let mut stack = WasmNetStack::new();
        let consensus = SwarmConsensus::new(&mut stack, 1, 9999);
        assert_eq!(consensus.state, SwarmState::Follower);
        assert_eq!(consensus.current_term, 0);
    }

    #[test]
    fn test_start_election() {
        let mut stack = WasmNetStack::new();
        let mut consensus = SwarmConsensus::new(&mut stack, 1, 9999);
        consensus.start_election(&mut stack);
        assert_eq!(consensus.state, SwarmState::Candidate);
        assert_eq!(consensus.current_term, 1);
        assert_eq!(consensus.voted_for, Some(1));
    }

    #[test]
    fn test_process_vote() {
        let mut stack = WasmNetStack::new();
        let mut consensus = SwarmConsensus::new(&mut stack, 1, 9999);
        consensus.start_election(&mut stack); // Becomes Candidate, term 1, 1 vote

        let vote_msg2 = ConsensusMessage::Vote {
            term: 1,
            voter_id: 2,
            vote_granted: true,
        };
        let vote_msg3 = ConsensusMessage::Vote {
            term: 1,
            voter_id: 3,
            vote_granted: true,
        };
        consensus.process_packet(&mut stack, &vote_msg2.serialize(), 4); // 4 nodes, needs > (4*2)/3 = 2. So 3 votes.
        assert_eq!(consensus.state, SwarmState::Candidate); // 2 votes, not enough
        consensus.process_packet(&mut stack, &vote_msg3.serialize(), 4);
        assert_eq!(consensus.state, SwarmState::Leader); // 3 votes, enough
    }

    #[test]
    fn test_process_request_vote_as_follower() {
        let mut stack = WasmNetStack::new();
        let mut consensus = SwarmConsensus::new(&mut stack, 2, 9999);

        let request = ConsensusMessage::RequestVote {
            term: 1,
            candidate_id: 1,
        };
        consensus.process_packet(&mut stack, &request.serialize(), 3);

        assert_eq!(consensus.current_term, 1);
        assert_eq!(consensus.voted_for, Some(1));
    }

    #[test]
    fn test_propose_task() {
        let mut stack = WasmNetStack::new();
        let mut consensus = SwarmConsensus::new(&mut stack, 1, 9999);
        consensus.state = SwarmState::Leader;

        assert!(consensus
            .propose_task(&mut stack, 42, alloc::vec![1, 2, 3])
            .is_ok());

        assert_eq!(consensus.task_prepares, alloc::vec![1]);
        assert_eq!(consensus.task_commits, alloc::vec![1]);
    }

    #[test]
    fn test_receive_task_proposal() {
        let mut stack = WasmNetStack::new();
        let mut consensus = SwarmConsensus::new(&mut stack, 2, 9999);

        let proposal = ConsensusMessage::ProposeTask {
            term: 1,
            leader_id: 1,
            task_id: 42,
            task_payload: alloc::vec![1, 2, 3],
        };

        consensus.process_packet(&mut stack, &proposal.serialize(), 3);

        assert_eq!(consensus.current_term, 1);
        assert_eq!(consensus.state, SwarmState::Follower);

        assert_eq!(consensus.task_prepares, alloc::vec![2]);
        assert_eq!(consensus.task_commits, alloc::vec![2]);

        // Proposal 2
        let proposal2 = ConsensusMessage::ProposeTask {
            term: 2,
            leader_id: 1,
            task_id: 43,
            task_payload: alloc::vec![1, 2, 3],
        };

        consensus.process_packet(&mut stack, &proposal2.serialize(), 3);

        assert_eq!(consensus.current_term, 2);
        assert_eq!(consensus.state, SwarmState::Follower);

        assert_eq!(consensus.task_prepares, alloc::vec![2]);
        assert_eq!(consensus.task_commits, alloc::vec![2]);
    }
}
