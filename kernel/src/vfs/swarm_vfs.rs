extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::cmp::min;

pub struct SwarmVfs {
    // node_id -> (chunk_id -> data)
    pub node_storage: BTreeMap<usize, BTreeMap<usize, Vec<u8>>>,
    // inode -> list of chunk_ids
    pub file_metadata: BTreeMap<u64, Vec<usize>>,
    pub chunk_size: usize,
    pub replication_factor: usize,
    pub next_chunk_id: usize,
    pub known_nodes: Vec<usize>,
}

impl Default for SwarmVfs {
    fn default() -> Self {
        Self::new(1024, 2, alloc::vec![1, 2, 3])
    }
}

impl SwarmVfs {
    pub fn new(chunk_size: usize, replication_factor: usize, known_nodes: Vec<usize>) -> Self {
        let mut node_storage = BTreeMap::new();
        for &node_id in &known_nodes {
            node_storage.insert(node_id, BTreeMap::new());
        }

        Self {
            node_storage,
            file_metadata: BTreeMap::new(),
            chunk_size,
            replication_factor,
            next_chunk_id: 0,
            known_nodes,
        }
    }

    pub fn write_file(&mut self, inode: u64, data: &[u8]) -> Result<(), &'static str> {
        if self.known_nodes.is_empty() {
            return Err("No nodes available in the swarm");
        }

        let mut chunks = Vec::new();
        let mut offset = 0;

        while offset < data.len() {
            let end = min(offset + self.chunk_size, data.len());
            let chunk_data = data[offset..end].to_vec();

            let chunk_id = self.next_chunk_id;
            self.next_chunk_id += 1;

            // Distribute chunk to `replication_factor` nodes
            for i in 0..self.replication_factor {
                let target_node_index = (chunk_id + i) % self.known_nodes.len();
                let target_node_id = self.known_nodes[target_node_index];

                if let Some(storage) = self.node_storage.get_mut(&target_node_id) {
                    storage.insert(chunk_id, chunk_data.clone());
                }
            }

            chunks.push(chunk_id);
            offset += self.chunk_size;
        }

        self.file_metadata.insert(inode, chunks);
        Ok(())
    }

    pub fn read_file(&self, inode: u64) -> Result<Vec<u8>, &'static str> {
        let chunk_ids = self.file_metadata.get(&inode).ok_or("File not found")?;
        let mut file_data = Vec::new();

        for &chunk_id in chunk_ids {
            let mut chunk_found = false;

            // Try to find the chunk in any known node
            for &node_id in &self.known_nodes {
                if let Some(storage) = self.node_storage.get(&node_id) {
                    if let Some(chunk_data) = storage.get(&chunk_id) {
                        file_data.extend_from_slice(chunk_data);
                        chunk_found = true;
                        break;
                    }
                }
            }

            if !chunk_found {
                return Err("Failed to retrieve chunk: data loss");
            }
        }

        Ok(file_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swarm_vfs_init() {
        let vfs = SwarmVfs::new(1024, 2, alloc::vec![1, 2, 3]);
        assert_eq!(vfs.chunk_size, 1024);
        assert_eq!(vfs.replication_factor, 2);
        assert_eq!(vfs.known_nodes, alloc::vec![1, 2, 3]);
        assert_eq!(vfs.node_storage.len(), 3);
    }

    #[test]
    fn test_swarm_vfs_write_and_read() {
        let mut vfs = SwarmVfs::new(4, 2, alloc::vec![1, 2, 3]);
        let data = b"hello world!"; // 12 bytes, so 3 chunks of 4 bytes

        assert!(vfs.write_file(1, data).is_ok());

        // Metadata should reflect 3 chunks
        assert_eq!(vfs.file_metadata.get(&1).unwrap().len(), 3);

        // Replication should put each chunk in 2 nodes
        let chunk_0_copies = vfs.known_nodes.iter().filter(|&node_id| {
            vfs.node_storage.get(node_id).unwrap().contains_key(&0)
        }).count();
        assert_eq!(chunk_0_copies, 2);

        // Read the file back
        let read_data = vfs.read_file(1).unwrap();
        assert_eq!(read_data, data);
    }

    #[test]
    fn test_swarm_vfs_file_not_found() {
        let vfs = SwarmVfs::default();
        assert!(vfs.read_file(99).is_err());
    }

    #[test]
    fn test_swarm_vfs_no_nodes() {
        let mut vfs = SwarmVfs::new(1024, 2, alloc::vec![]);
        assert!(vfs.write_file(1, b"test").is_err());
    }
}
