#![no_std]

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use vector_db::{VectorDb, VectorRecord};
use inference_runtime::{InferenceEngine, Model, Tensor};

pub struct SemanticLogger {
    db: VectorDb,
    engine: InferenceEngine,
    model: Model,
    next_id: usize,
}

impl SemanticLogger {
    pub fn new() -> Result<Self, &'static str> {
        let mut engine = InferenceEngine::new();
        let model = engine
            .load_model_by_name("embedding_model")
            .map_err(|_| "Failed to load embedding model")?;

        Ok(Self {
            db: VectorDb::new(),
            engine,
            model,
            next_id: 1,
        })
    }

    fn generate_embedding(&mut self, text: &str) -> Result<Vec<f32>, &'static str> {
        let ctx = self
            .engine
            .init_execution_context(&self.model)
            .map_err(|_| "Failed to init execution context")?;

        let data = vec![0; text.len()];
        let tensor = Tensor::new(data, vec![text.len()]);

        self.engine
            .set_input(ctx, 0, &tensor)
            .map_err(|_| "Failed to set input")?;

        self.engine.compute(ctx).map_err(|_| "Failed to compute")?;

        let mut out_buffer = [0u8; 12];
        let _ = self
            .engine
            .get_output(ctx, 0, &mut out_buffer)
            .map_err(|_| "Failed to get output")?;

        // Mock embedding generation based on string length, similar to InputHandler in wgpu_compositor
        let val = text.len() as f32;
        Ok(vec![val, val * 0.5, val * 2.0])
    }

    pub fn log(&mut self, level: &str, message: &str) -> Result<(), &'static str> {
        let content = alloc::format!("[{}] {}", level, message);
        let embedding = self.generate_embedding(&content)?;

        let id = self.next_id.to_string();
        self.next_id += 1;

        let record = VectorRecord {
            id,
            vector: embedding,
            metadata: Some(content),
        };

        self.db
            .insert(record)
            .map_err(|_| "Failed to insert into vector DB")?;
        Ok(())
    }

    pub fn query(&mut self, query_text: &str, k: usize) -> Result<Vec<(f32, String)>, &'static str> {
        if query_text.is_empty() {
            return Ok(Vec::new());
        }

        let query_embedding = self.generate_embedding(query_text)?;

        let results = self
            .db
            .search_cosine(&query_embedding, k)
            .map_err(|_| "Failed to search vector DB")?;

        let mut final_results = Vec::new();
        for (score, record) in results {
            if let Some(meta) = &record.metadata {
                final_results.push((score, meta.clone()));
            } else {
                final_results.push((score, record.id.clone()));
            }
        }

        Ok(final_results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_logger_creation() {
        let logger = SemanticLogger::new();
        assert!(logger.is_ok());
    }

    #[test]
    fn test_semantic_logger_log_and_query() {
        let mut logger = SemanticLogger::new().unwrap();

        logger.log("ERROR", "Kernel panic: divide by zero").unwrap();
        logger.log("WARN", "Disk space is running low").unwrap();

        let results = logger.query("panic", 1).unwrap();
        assert!(!results.is_empty());
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
static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

#[cfg(not(test))]
unsafe impl GlobalAlloc for SimpleAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let align = layout.align();
        let size = layout.size();
        let offset = self.offset.load(Ordering::SeqCst);
        let res = offset.next_multiple_of(align);
        let next_offset = res + size;
        if next_offset > HEAP_SIZE {
            core::ptr::null_mut()
        } else {
            self.offset.store(next_offset, Ordering::SeqCst);
            #[allow(static_mut_refs)]
            HEAP.as_mut_ptr().add(res)
        }
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[cfg(not(test))]
#[global_allocator]
static ALLOCATOR: SimpleAllocator = SimpleAllocator {
    offset: AtomicUsize::new(0),
};
