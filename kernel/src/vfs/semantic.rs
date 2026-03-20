extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use inference_runtime::{InferenceEngine, Model, Tensor};
use vector_db::{VectorDb, VectorRecord};

pub struct SemanticSearch {
    db: VectorDb,
    engine: InferenceEngine,
    model: Model,
}

impl SemanticSearch {
    pub fn new() -> Result<Self, &'static str> {
        let mut engine = InferenceEngine::new();
        // Load a mock model for generating embeddings
        let model = engine
            .load_model_by_name("embedding_model")
            .map_err(|_| "Failed to load embedding model")?;

        Ok(Self {
            db: VectorDb::new(),
            engine,
            model,
        })
    }

    fn generate_embedding(&mut self, text: &str) -> Result<Vec<f32>, &'static str> {
        let ctx = self
            .engine
            .init_execution_context(&self.model)
            .map_err(|_| "Failed to init execution context")?;

        // Use the text length as a dummy tensor for the mock engine
        let data = alloc::vec![0; text.len()];
        let tensor = Tensor::new(data, alloc::vec![text.len()]);

        self.engine
            .set_input(ctx, 0, &tensor)
            .map_err(|_| "Failed to set input")?;

        self.engine.compute(ctx).map_err(|_| "Failed to compute")?;

        let mut out_buffer = [0u8; 12]; // We'll mock a 3D float vector (3 * 4 bytes)
        let _ = self
            .engine
            .get_output(ctx, 0, &mut out_buffer)
            .map_err(|_| "Failed to get output")?;

        // In reality, we'd parse the output buffer into floats.
        // For the mock, we'll just generate a dummy vector based on the text length.
        let val = text.len() as f32;
        Ok(alloc::vec![val, val * 0.5, val * 2.0])
    }

    pub fn index_file(&mut self, inode: u64, content: &str) -> Result<(), &'static str> {
        let embedding = self.generate_embedding(content)?;

        let record = VectorRecord {
            id: alloc::format!("{}", inode),
            vector: embedding,
            metadata: Some(alloc::string::String::from(content)),
        };

        self.db
            .insert(record)
            .map_err(|_| "Failed to insert into vector DB")?;
        Ok(())
    }

    pub fn search(&mut self, query: &str, k: usize) -> Result<Vec<(f32, u64)>, &'static str> {
        let query_embedding = self.generate_embedding(query)?;

        let results = self
            .db
            .search_cosine(&query_embedding, k)
            .map_err(|_| "Failed to search vector DB")?;

        let mut final_results = Vec::new();
        for (score, record) in results {
            if let Ok(inode) = record.id.parse::<u64>() {
                final_results.push((score, inode));
            }
        }

        Ok(final_results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_search_init() {
        let search = SemanticSearch::new();
        assert!(search.is_ok());
    }

    #[test]
    fn test_semantic_search_index_and_search() {
        let mut search = SemanticSearch::new().unwrap();

        // Index some files
        assert!(search.index_file(1, "system configuration file").is_ok());
        assert!(search.index_file(2, "user password database").is_ok());
        assert!(search.index_file(3, "network interface settings").is_ok());

        // Search
        let results = search.search("config", 2).unwrap();
        assert_eq!(results.len(), 2);
    }
}
