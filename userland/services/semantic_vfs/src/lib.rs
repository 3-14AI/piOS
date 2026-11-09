#![no_std]

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use inference_runtime::{InferenceEngine, Tensor};
use vector_db::{VectorDb, VectorRecord};

pub struct SemanticVfs {
    pub engine: InferenceEngine,
    pub db: VectorDb,
    pub files: BTreeMap<String, String>,
}

impl Default for SemanticVfs {
    fn default() -> Self {
        Self::new()
    }
}

impl SemanticVfs {
    pub fn new() -> Self {
        Self {
            engine: InferenceEngine::new(),
            db: VectorDb::new(),
            files: BTreeMap::new(),
        }
    }

    fn generate_embedding(&mut self, text: &str) -> Option<Vec<f32>> {
        let model = self.engine.load_model_by_name("semantic_vfs").ok()?;
        let ctx = self.engine.init_execution_context(&model).ok()?;

        let bytes = text.as_bytes().to_vec();
        let tensor = Tensor::new(bytes.clone(), alloc::vec![bytes.len()]);

        if self.engine.set_input(ctx, 0, &tensor).is_ok() && self.engine.compute(ctx).is_ok() {
            let mut out_buffer = alloc::vec![0u8; 128];
            if let Ok(bytes_written) = self.engine.get_output(ctx, 0, &mut out_buffer) {
                // Mock embedding conversion: sum chars and create a normalized vector
                let sum: f32 = out_buffer[..bytes_written].iter().map(|&b| b as f32).sum();
                let len = if bytes.is_empty() {
                    1.0
                } else {
                    bytes.len() as f32
                };
                return Some(alloc::vec![sum / len, (sum % 10.0) / 10.0, 0.5]);
            }
        }

        // Fallback simple embedding logic for testing
        let mut sum = 0.0;
        for c in text.chars() {
            sum += c as u32 as f32;
        }
        Some(alloc::vec![sum / 100.0, (sum % 10.0) / 10.0, 0.5])
    }

    pub fn insert_file(&mut self, path: &str, content: &str) -> bool {
        self.files.insert(path.to_string(), content.to_string());

        if let Some(embedding) = self.generate_embedding(content) {
            let record = VectorRecord {
                id: path.to_string(),
                vector: embedding,
                metadata: None,
            };
            self.db.insert(record).is_ok()
        } else {
            false
        }
    }

    pub fn read_file(&self, path: &str) -> Option<&String> {
        self.files.get(path)
    }

    pub fn search_semantic(&mut self, query: &str, top_k: usize) -> Vec<(f32, String)> {
        if let Some(embedding) = self.generate_embedding(query) {
            let results_ok = self.db.search_cosine(&embedding, top_k);
            if let Ok(results) = results_ok {
                return results
                    .into_iter()
                    .map(|(score, record)| (score, record.id.clone()))
                    .collect();
            }
        }
        alloc::vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_read_file() {
        let mut vfs = SemanticVfs::new();
        assert!(vfs.insert_file("/docs/test.txt", "Hello World"));

        let content = vfs.read_file("/docs/test.txt");
        assert_eq!(content.unwrap(), "Hello World");
    }

    #[test]
    fn test_search_semantic() {
        let mut vfs = SemanticVfs::new();
        vfs.insert_file("/docs/budget_2023.txt", "Budget report for 2023");
        vfs.insert_file(
            "/docs/meeting_notes.txt",
            "Notes from the marketing meeting",
        );

        let results = vfs.search_semantic("Budget report for 2023", 1);
        assert!(!results.is_empty());
        assert_eq!(results[0].1, "/docs/budget_2023.txt");
    }
}
