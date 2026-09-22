use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FederatedMessage {
    GradientUpdate {
        model_id: usize,
        node_id: usize,
        weights: Vec<u8>,
    },
    ModelSync {
        model_id: usize,
        version: usize,
        global_weights: Vec<u8>,
    },
}

impl FederatedMessage {
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            FederatedMessage::GradientUpdate {
                model_id,
                node_id,
                weights,
            } => {
                let mut data = alloc::vec![1];
                data.extend_from_slice(&model_id.to_le_bytes());
                data.extend_from_slice(&node_id.to_le_bytes());
                data.extend_from_slice(weights);
                data
            }
            FederatedMessage::ModelSync {
                model_id,
                version,
                global_weights,
            } => {
                let mut data = alloc::vec![2];
                data.extend_from_slice(&model_id.to_le_bytes());
                data.extend_from_slice(&version.to_le_bytes());
                data.extend_from_slice(global_weights);
                data
            }
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.is_empty() {
            return None;
        }

        match bytes[0] {
            1 => {
                if bytes.len() < 17 {
                    return None;
                }
                let mut model_bytes = [0u8; 8];
                model_bytes.copy_from_slice(&bytes[1..9]);
                let model_id = usize::from_le_bytes(model_bytes);

                let mut node_bytes = [0u8; 8];
                node_bytes.copy_from_slice(&bytes[9..17]);
                let node_id = usize::from_le_bytes(node_bytes);

                let weights = bytes[17..].to_vec();

                Some(FederatedMessage::GradientUpdate {
                    model_id,
                    node_id,
                    weights,
                })
            }
            2 => {
                if bytes.len() < 17 {
                    return None;
                }
                let mut model_bytes = [0u8; 8];
                model_bytes.copy_from_slice(&bytes[1..9]);
                let model_id = usize::from_le_bytes(model_bytes);

                let mut version_bytes = [0u8; 8];
                version_bytes.copy_from_slice(&bytes[9..17]);
                let version = usize::from_le_bytes(version_bytes);

                let global_weights = bytes[17..].to_vec();

                Some(FederatedMessage::ModelSync {
                    model_id,
                    version,
                    global_weights,
                })
            }
            _ => None,
        }
    }
}

pub struct FederatedNode {
    pub node_id: usize,
    pub is_leader: bool,
    pub collected_gradients: Vec<Vec<u8>>,
    pub global_model_version: usize,
}

impl Default for FederatedNode {
    fn default() -> Self {
        Self::new(1, false)
    }
}

impl FederatedNode {
    pub fn new(node_id: usize, is_leader: bool) -> Self {
        Self {
            node_id,
            is_leader,
            collected_gradients: Vec::new(),
            global_model_version: 0,
        }
    }

    pub fn handle_message(&mut self, msg: FederatedMessage) {
        match msg {
            FederatedMessage::GradientUpdate { weights, .. } => {
                if self.is_leader {
                    self.collected_gradients.push(weights);
                }
            }
            FederatedMessage::ModelSync { version, .. } => {
                if !self.is_leader && version > self.global_model_version {
                    self.global_model_version = version;
                    // Apply weights... (mocked)
                }
            }
        }
    }

    pub fn aggregate_gradients(&mut self) -> Option<Vec<u8>> {
        if !self.is_leader || self.collected_gradients.is_empty() {
            return None;
        }

        // Mock aggregation: just average/merge byte values
        let mut aggregated = self.collected_gradients[0].clone();
        for weights in self.collected_gradients.iter().skip(1) {
            for (i, w) in weights.iter().enumerate() {
                if i < aggregated.len() {
                    aggregated[i] = aggregated[i].wrapping_add(*w) / 2;
                }
            }
        }

        self.collected_gradients.clear();
        self.global_model_version += 1;
        Some(aggregated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_federated_message_serialization() {
        let update = FederatedMessage::GradientUpdate {
            model_id: 1,
            node_id: 2,
            weights: alloc::vec![1, 2, 3],
        };
        let bytes = update.to_bytes();
        let decoded = FederatedMessage::from_bytes(&bytes).unwrap();
        assert_eq!(update, decoded);

        let sync = FederatedMessage::ModelSync {
            model_id: 1,
            version: 5,
            global_weights: alloc::vec![4, 5, 6],
        };
        let bytes_sync = sync.to_bytes();
        let decoded_sync = FederatedMessage::from_bytes(&bytes_sync).unwrap();
        assert_eq!(sync, decoded_sync);
    }

    #[test]
    fn test_federated_node_logic() {
        let mut leader = FederatedNode::new(1, true);
        let mut worker = FederatedNode::new(2, false);

        // Worker sends update
        let update = FederatedMessage::GradientUpdate {
            model_id: 1,
            node_id: 2,
            weights: alloc::vec![10, 20, 30],
        };
        leader.handle_message(update);
        assert_eq!(leader.collected_gradients.len(), 1);

        // Leader aggregates
        let new_weights = leader.aggregate_gradients().unwrap();
        assert_eq!(new_weights, alloc::vec![10, 20, 30]);
        assert_eq!(leader.global_model_version, 1);

        // Leader syncs to worker
        let sync = FederatedMessage::ModelSync {
            model_id: 1,
            version: leader.global_model_version,
            global_weights: new_weights,
        };
        worker.handle_message(sync);
        assert_eq!(worker.global_model_version, 1);
    }
}
