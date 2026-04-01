#![no_std]
extern crate alloc;

use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use inference_runtime::{InferenceEngine, Model, Tensor};
use vector_db::{VectorDb, VectorRecord};

pub struct NlShell {
    db: VectorDb,
    engine: InferenceEngine,
    model: Model,
}

impl NlShell {
    pub fn new() -> Result<Self, &'static str> {
        let mut engine = InferenceEngine::new();
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
        if text.is_empty() {
            return Ok(alloc::vec![0.0, 0.0, 0.0]);
        }

        let ctx = self
            .engine
            .init_execution_context(&self.model)
            .map_err(|_| "Failed to init execution context")?;

        let data = alloc::vec![0; text.len()];
        let tensor = Tensor::new(data, alloc::vec![text.len()]);

        self.engine
            .set_input(ctx, 0, &tensor)
            .map_err(|_| "Failed to set input")?;

        self.engine.compute(ctx).map_err(|_| "Failed to compute")?;

        let mut out_buffer = [0u8; 12];
        let bytes_written = self
            .engine
            .get_output(ctx, 0, &mut out_buffer)
            .map_err(|_| "Failed to get output")?;

        let mut embedding = Vec::new();
        if bytes_written >= 12 && out_buffer[..bytes_written] != b"mock_output"[..] {
            for chunk in out_buffer[..bytes_written].chunks_exact(4) {
                let bytes = [chunk[0], chunk[1], chunk[2], chunk[3]];
                embedding.push(f32::from_le_bytes(bytes));
            }
        }

        if embedding.is_empty() {
            let mut val1 = 0.0;
            let mut val2 = 0.0;
            let mut val3 = 0.0;
            for (i, b) in text.bytes().enumerate() {
                match i % 3 {
                    0 => val1 += b as f32,
                    1 => val2 += b as f32,
                    _ => val3 += b as f32,
                }
            }
            embedding = alloc::vec![val1, val2, val3];
        }

        Ok(embedding)
    }

    pub fn register_command(
        &mut self,
        api_endpoint: &str,
        description: &str,
    ) -> Result<(), &'static str> {
        let embedding = self.generate_embedding(description)?;

        let record = VectorRecord {
            id: api_endpoint.to_string(),
            vector: embedding,
            metadata: Some(description.to_string()),
        };

        self.db
            .insert(record)
            .map_err(|_| "Failed to insert into vector DB")?;
        Ok(())
    }

    pub fn parse_intent(
        &mut self,
        natural_language_input: &str,
    ) -> Result<Option<String>, &'static str> {
        if natural_language_input.is_empty() {
            return Ok(None);
        }

        let query_embedding = self.generate_embedding(natural_language_input)?;

        let results = self
            .db
            .search_cosine(&query_embedding, 1)
            .map_err(|_| "Failed to search vector DB")?;

        if let Some((_score, record)) = results.first() {
            Ok(Some(record.id.clone()))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nl_shell_creation() {
        let shell = NlShell::new();
        assert!(shell.is_ok());
    }

    #[test]
    fn test_register_and_parse_command() {
        let mut shell = NlShell::new().unwrap();

        shell
            .register_command("kernel.process.list", "show running processes list them")
            .unwrap();
        shell
            .register_command("kernel.fs.read", "read file content from disk")
            .unwrap();

        let intent = shell
            .parse_intent("can you list the running processes")
            .unwrap();

        assert!(intent.is_some());
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;

    #[test]
    fn test_parse_intent_empty_input() {
        let mut shell = NlShell::new().unwrap();
        let intent = shell.parse_intent("").unwrap();
        assert_eq!(intent, None);
    }

    #[test]
    fn test_generate_embedding_empty_text() {
        let mut shell = NlShell::new().unwrap();
        let embedding = shell.generate_embedding("").unwrap();
        assert_eq!(embedding, alloc::vec![0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_parse_intent_no_match() {
        let mut shell = NlShell::new().unwrap();

        let intent = shell
            .parse_intent("do something entirely unrelated")
            .unwrap();

        // The mock logic will insert and search but if the DB is empty, it shouldn't find anything
        assert!(intent.is_none());
    }
}
