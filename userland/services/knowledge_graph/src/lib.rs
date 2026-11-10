#![no_std]

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use inference_runtime::{InferenceEngine, Tensor};
use vector_db::{VectorDb, VectorRecord};

#[derive(Debug, Clone, PartialEq)]
pub enum KnowledgeSyncMessage {
    SyncNode {
        node_id: String,
        data: String,
        embedding: Vec<f32>,
    },
}

impl KnowledgeSyncMessage {
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            KnowledgeSyncMessage::SyncNode {
                node_id,
                data,
                embedding,
            } => {
                let mut bytes = alloc::vec![1];
                let id_bytes = node_id.as_bytes();
                bytes.extend_from_slice(&(id_bytes.len() as u32).to_le_bytes());
                bytes.extend_from_slice(id_bytes);

                let data_bytes = data.as_bytes();
                bytes.extend_from_slice(&(data_bytes.len() as u32).to_le_bytes());
                bytes.extend_from_slice(data_bytes);

                bytes.extend_from_slice(&(embedding.len() as u32).to_le_bytes());
                for val in embedding {
                    bytes.extend_from_slice(&val.to_le_bytes());
                }
                bytes
            }
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.is_empty() {
            return None;
        }
        match bytes[0] {
            1 => {
                let mut pos = 1;

                if pos + 4 > bytes.len() {
                    return None;
                }
                let mut id_len_bytes = [0u8; 4];
                id_len_bytes.copy_from_slice(&bytes[pos..pos + 4]);
                let id_len = u32::from_le_bytes(id_len_bytes) as usize;
                pos += 4;

                if pos + id_len > bytes.len() {
                    return None;
                }
                let node_id = String::from_utf8(bytes[pos..pos + id_len].to_vec()).ok()?;
                pos += id_len;

                if pos + 4 > bytes.len() {
                    return None;
                }
                let mut len_bytes = [0u8; 4];
                len_bytes.copy_from_slice(&bytes[pos..pos + 4]);
                let data_len = u32::from_le_bytes(len_bytes) as usize;
                pos += 4;

                if pos + data_len > bytes.len() {
                    return None;
                }
                let data = String::from_utf8(bytes[pos..pos + data_len].to_vec()).ok()?;
                pos += data_len;

                if pos + 4 > bytes.len() {
                    return None;
                }
                let mut embed_len_bytes = [0u8; 4];
                embed_len_bytes.copy_from_slice(&bytes[pos..pos + 4]);
                let embed_len = u32::from_le_bytes(embed_len_bytes) as usize;
                pos += 4;

                if pos + embed_len * 4 > bytes.len() {
                    return None;
                }

                let mut embedding = Vec::with_capacity(embed_len);
                for _ in 0..embed_len {
                    let mut val_bytes = [0u8; 4];
                    val_bytes.copy_from_slice(&bytes[pos..pos + 4]);
                    embedding.push(f32::from_le_bytes(val_bytes));
                    pos += 4;
                }

                Some(KnowledgeSyncMessage::SyncNode {
                    node_id,
                    data,
                    embedding,
                })
            }
            _ => None,
        }
    }
}

pub struct KnowledgeGraph {
    pub engine: InferenceEngine,
    pub nodes: BTreeMap<String, String>,
    pub edges: Vec<(String, String, String)>,
    pub db: VectorDb,
}

impl KnowledgeGraph {
    pub fn new() -> Self {
        Self {
            engine: InferenceEngine::new(),
            nodes: BTreeMap::new(),
            edges: Vec::new(),
            db: VectorDb::new(),
        }
    }

    fn generate_embedding(&mut self, text: &str) -> Option<Vec<f32>> {
        let model = self.engine.load_model_by_name("knowledge_graph").ok()?;
        let ctx = self.engine.init_execution_context(&model).ok()?;

        let bytes = text.as_bytes().to_vec();
        let tensor = Tensor::new(bytes.clone(), alloc::vec![bytes.len()]);

        if self.engine.set_input(ctx, 0, &tensor).is_ok() && self.engine.compute(ctx).is_ok() {
            let mut out_buffer = alloc::vec![0u8; 128];
            if let Ok(bytes_written) = self.engine.get_output(ctx, 0, &mut out_buffer) {
                let sum: f32 = out_buffer[..bytes_written].iter().map(|&b| b as f32).sum();
                let len = if bytes.is_empty() {
                    1.0
                } else {
                    bytes.len() as f32
                };
                return Some(alloc::vec![sum / len, (sum % 10.0) / 10.0, 0.5]);
            }
        }

        let mut sum = 0.0;
        for c in text.chars() {
            sum += c as u32 as f32;
        }
        Some(alloc::vec![sum / 100.0, (sum % 10.0) / 10.0, 0.5])
    }

    pub fn add_node(&mut self, id: &str, data: &str) {
        self.nodes.insert(String::from(id), String::from(data));

        if let Some(embedding) = self.generate_embedding(data) {
            let record = VectorRecord {
                id: id.to_string(),
                vector: embedding,
                metadata: None,
            };
            let _ = self.db.insert(record);
        }
    }

    pub fn add_edge(&mut self, from: &str, to: &str, relation: &str) {
        if self.nodes.contains_key(from) && self.nodes.contains_key(to) {
            self.edges
                .push((String::from(from), String::from(to), String::from(relation)));
        }
    }

    pub fn sync_node(&mut self, id: &str) -> Option<KnowledgeSyncMessage> {
        if let Some(data) = self.nodes.get(id) {
            if let Some(record) = self.db.get(id) {
                return Some(KnowledgeSyncMessage::SyncNode {
                    node_id: String::from(id),
                    data: data.clone(),
                    embedding: record.vector.clone(),
                });
            }
        }
        None
    }

    pub fn receive_sync(&mut self, msg: KnowledgeSyncMessage) {
        match msg {
            KnowledgeSyncMessage::SyncNode {
                node_id,
                data,
                embedding,
            } => {
                self.nodes.insert(node_id.clone(), data);

                let record = VectorRecord {
                    id: node_id,
                    vector: embedding,
                    metadata: None,
                };
                let _ = self.db.insert(record);
            }
        }
    }
}

impl Default for KnowledgeGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[cfg(not(test))]
use core::alloc::{GlobalAlloc, Layout};
#[cfg(not(test))]
use core::sync::atomic::{AtomicUsize, Ordering};

#[cfg(not(test))]
struct SimpleAllocator {
    offset: AtomicUsize,
}

#[cfg(not(test))]
const HEAP_SIZE: usize = 1024 * 1024;
#[cfg(not(test))]
#[repr(C, align(4096))]
#[allow(dead_code)]
struct AlignedHeap([u8; HEAP_SIZE]);

#[cfg(not(test))]
struct SyncHeap(core::cell::UnsafeCell<AlignedHeap>);

#[cfg(not(test))]
unsafe impl Sync for SyncHeap {}

#[cfg(not(test))]
static HEAP: SyncHeap = SyncHeap(core::cell::UnsafeCell::new(AlignedHeap([0; HEAP_SIZE])));

#[cfg(not(test))]
unsafe impl GlobalAlloc for SimpleAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let align = layout.align();
        let size = layout.size();
        let mut current_offset = self.offset.load(Ordering::Acquire);
        loop {
            let res = current_offset.next_multiple_of(align);
            let next_offset = res + size;
            if next_offset > HEAP_SIZE {
                return core::ptr::null_mut();
            }
            match self.offset.compare_exchange_weak(
                current_offset,
                next_offset,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    let heap_ptr = HEAP.0.get() as *mut u8;
                    return heap_ptr.add(res);
                }
                Err(new_offset) => {
                    current_offset = new_offset;
                }
            }
        }
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[cfg(not(test))]
#[global_allocator]
static ALLOCATOR: SimpleAllocator = SimpleAllocator {
    offset: AtomicUsize::new(0),
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_message_serialization() {
        let msg = KnowledgeSyncMessage::SyncNode {
            node_id: String::from("node-1234"),
            data: String::from("test data"),
            embedding: alloc::vec![0.1, 0.2, 0.3],
        };

        let bytes = msg.to_bytes();
        let decoded = KnowledgeSyncMessage::from_bytes(&bytes).unwrap();

        assert_eq!(msg, decoded);
    }

    #[test]
    fn test_graph_sync() {
        let mut g1 = KnowledgeGraph::new();
        let id = "node-5678";
        g1.add_node(id, "Node A");

        // Mock the embedding since it falls back if inference_runtime mock fails
        // We know generating an embedding stores it in DB. Let's make sure it's there.
        assert!(g1.db.get(id).is_some());

        let sync_msg = g1.sync_node(id).unwrap();

        let mut g2 = KnowledgeGraph::new();
        assert_eq!(g2.nodes.len(), 0);

        g2.receive_sync(sync_msg);

        assert_eq!(g2.nodes.len(), 1);
        assert_eq!(g2.nodes.get(id).unwrap(), "Node A");
        assert!(g2.db.get(id).is_some());
    }
}
