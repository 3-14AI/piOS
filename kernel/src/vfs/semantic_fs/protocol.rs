extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::convert::TryInto;
use crate::vfs::semantic::SemanticSearch;

// Represents the Universal Semantic File Protocol (USFP) message structure
#[derive(Debug, PartialEq)]
pub enum UsfpMessage {
    // Client -> Server
    MountRequest { version: u32, client_id: String },
    QueryRequest { query: String, max_results: u32 },
    // Server -> Client
    MountResponse { status: u32 },
    QueryResponse { results: Vec<(u64, f32)> },
}

pub struct UsfpServer {
    pub search_engine: SemanticSearch,
    pub is_mounted: bool,
}

impl UsfpServer {
    pub fn new() -> Result<Self, &'static str> {
        Ok(Self {
            search_engine: SemanticSearch::new()?,
            is_mounted: false,
        })
    }

    pub fn handle_packet(&mut self, data: &[u8]) -> Option<Vec<u8>> {
        let msg = Self::deserialize(data)?;

        let response = match msg {
            UsfpMessage::MountRequest { version, .. } => {
                if version >= 1 {
                    self.is_mounted = true;
                    UsfpMessage::MountResponse { status: 0 } // Success
                } else {
                    UsfpMessage::MountResponse { status: 1 } // Error: Unsupported version
                }
            },
            UsfpMessage::QueryRequest { query, max_results } => {
                if !self.is_mounted {
                    return None; // Unauthorized
                }

                let search_results = self.search_engine.search(&query, max_results as usize).unwrap_or_default();
                let mut results = Vec::new();
                for (score, inode) in search_results {
                    results.push((inode, score));
                }

                UsfpMessage::QueryResponse { results }
            },
            _ => return None, // Ignore server->client messages sent to server
        };

        Some(Self::serialize(response))
    }

    // Basic bare-metal compatible serialization
    fn serialize(msg: UsfpMessage) -> Vec<u8> {
        let mut buf = Vec::new();
        match msg {
            UsfpMessage::MountResponse { status } => {
                buf.push(0x01); // Type: MountResponse
                buf.extend_from_slice(&status.to_le_bytes());
            },
            UsfpMessage::QueryResponse { results } => {
                buf.push(0x02); // Type: QueryResponse
                buf.extend_from_slice(&(results.len() as u32).to_le_bytes());
                for (inode, score) in results {
                    buf.extend_from_slice(&inode.to_le_bytes());
                    buf.extend_from_slice(&score.to_le_bytes());
                }
            },
            _ => {},
        }
        buf
    }

    fn deserialize(data: &[u8]) -> Option<UsfpMessage> {
        if data.is_empty() { return None; }

        match data[0] {
            0x11 => { // Type: MountRequest
                if data.len() < 5 { return None; }
                let version = u32::from_le_bytes(data[1..5].try_into().unwrap());
                let client_id = core::str::from_utf8(&data[5..]).ok()?.to_string();
                Some(UsfpMessage::MountRequest { version, client_id })
            },
            0x12 => { // Type: QueryRequest
                if data.len() < 5 { return None; }
                let max_results = u32::from_le_bytes(data[1..5].try_into().unwrap());
                let query = core::str::from_utf8(&data[5..]).ok()?.to_string();
                Some(UsfpMessage::QueryRequest { query, max_results })
            },
            _ => None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usfp_mount() {
        let mut server = UsfpServer::new().unwrap();

        let mut mount_req = alloc::vec![0x11];
        mount_req.extend_from_slice(&1u32.to_le_bytes()); // Version 1
        mount_req.extend_from_slice(b"legacy-linux");

        let resp_bytes = server.handle_packet(&mount_req).unwrap();
        assert_eq!(resp_bytes, alloc::vec![0x01, 0, 0, 0, 0]); // Status 0
        assert!(server.is_mounted);
    }

    #[test]
    fn test_usfp_query() {
        let mut server = UsfpServer::new().unwrap();
        server.is_mounted = true;
        server.search_engine.index_file(42, "test semantic file content").unwrap();

        let mut query_req = alloc::vec![0x12];
        query_req.extend_from_slice(&1u32.to_le_bytes()); // max_results = 1
        query_req.extend_from_slice(b"test");

        let resp_bytes = server.handle_packet(&query_req).unwrap();
        assert_eq!(resp_bytes[0], 0x02); // Type: QueryResponse
        // We just ensure it processed and returned results (which depend on the mock embedding)
        assert!(resp_bytes.len() > 5);
    }
}
