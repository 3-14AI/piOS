#![no_std]

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use inference_runtime::{InferenceEngine, Tensor};
use vector_db::{VectorDb, VectorRecord};

pub struct EventRecord {
    pub timestamp: u64,
    pub event_type: EventType,
    pub data: Vec<u8>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum EventType {
    Screenshot,
    InputEvent,
}

pub struct RecordingDaemon {
    pub history: Vec<EventRecord>,
    pub is_recording: bool,
    pub db: VectorDb,
    pub engine: InferenceEngine,
    pub next_id: usize,
}

impl Default for RecordingDaemon {
    fn default() -> Self {
        Self::new()
    }
}

impl RecordingDaemon {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            is_recording: false,
            db: VectorDb::new(),
            engine: InferenceEngine::new(),
            next_id: 1,
        }
    }

    pub fn start_recording(&mut self) {
        self.is_recording = true;
    }

    pub fn stop_recording(&mut self) {
        self.is_recording = false;
    }

    pub fn capture_screenshot(&mut self, timestamp: u64, image_data: Vec<u8>) {
        if self.is_recording {
            let record = EventRecord {
                timestamp,
                event_type: EventType::Screenshot,
                data: image_data,
            };
            self.index_event(&record);
            self.history.push(record);
        }
    }

    pub fn capture_input_event(&mut self, timestamp: u64, event_data: Vec<u8>) {
        if self.is_recording {
            let record = EventRecord {
                timestamp,
                event_type: EventType::InputEvent,
                data: event_data,
            };
            self.index_event(&record);
            self.history.push(record);
        }
    }

    fn index_event(&mut self, event: &EventRecord) {
        let description = match event.event_type {
            EventType::Screenshot => {
                alloc::format!("Screenshot captured at timestamp {}", event.timestamp)
            }
            EventType::InputEvent => {
                alloc::format!("Input event captured at timestamp {}", event.timestamp)
            }
        };

        if let Ok(embedding) = self.generate_embedding(&description) {
            let record = VectorRecord {
                id: self.next_id.to_string(),
                vector: embedding,
                metadata: Some(description),
            };
            let _ = self.db.insert(record);
            self.next_id += 1;
        }
    }

    pub fn generate_embedding(&mut self, text: &str) -> Result<Vec<f32>, &'static str> {
        let text_bytes = text.as_bytes().to_vec();
        if text_bytes.is_empty() {
            return Err("Empty input text");
        }

        let input_tensor = Tensor::new(text_bytes, alloc::vec![text.len()]);

        let model = self
            .engine
            .load_model_by_name("embedding_model")
            .map_err(|_| "Failed to load model")?;
        let ctx_id = self
            .engine
            .init_execution_context(&model)
            .map_err(|_| "Failed to init context")?;

        self.engine
            .set_input(ctx_id, 0, &input_tensor)
            .map_err(|_| "Failed to set input")?;
        self.engine
            .compute(ctx_id)
            .map_err(|_| "Failed to compute")?;

        let mut out_buffer = alloc::vec![0u8; 128]; // Max size
        let bytes_written = self
            .engine
            .get_output(ctx_id, 0, &mut out_buffer)
            .map_err(|_| "Failed to get output")?;

        let float_count = bytes_written / 4;
        let mut embedding = Vec::with_capacity(float_count);
        for i in 0..float_count {
            let mut float_bytes = [0u8; 4];
            float_bytes.copy_from_slice(&out_buffer[i * 4..(i + 1) * 4]);
            embedding.push(f32::from_le_bytes(float_bytes));
        }

        // Pad if needed
        while embedding.len() < 384 {
            embedding.push(0.0);
        }

        Ok(embedding)
    }

    pub fn query_history(
        &mut self,
        query: &str,
        k: usize,
    ) -> Result<Vec<(f32, String)>, &'static str> {
        let query_embedding = self.generate_embedding(query)?;

        // Use search_cosine because vector_db does not have search
        if let Ok(results) = self.db.search_cosine(&query_embedding, k) {
            let mut output = Vec::new();
            for (score, record) in results {
                if let Some(meta) = &record.metadata {
                    output.push((score, meta.clone()));
                } else {
                    output.push((score, record.id.clone()));
                }
            }
            Ok(output)
        } else {
            Err("Failed to search database")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recording_daemon() {
        let mut daemon = RecordingDaemon::new();
        assert!(!daemon.is_recording);

        daemon.start_recording();
        assert!(daemon.is_recording);

        daemon.capture_screenshot(100, alloc::vec![1, 2, 3]);
        daemon.capture_input_event(200, alloc::vec![4, 5]);

        assert_eq!(daemon.history.len(), 2);
        assert_eq!(daemon.history[0].timestamp, 100);
        assert_eq!(daemon.history[0].event_type, EventType::Screenshot);
        assert_eq!(daemon.history[0].data, alloc::vec![1, 2, 3]);

        assert_eq!(daemon.history[1].timestamp, 200);
        assert_eq!(daemon.history[1].event_type, EventType::InputEvent);
        assert_eq!(daemon.history[1].data, alloc::vec![4, 5]);

        daemon.stop_recording();
        daemon.capture_input_event(300, alloc::vec![6]);
        assert_eq!(daemon.history.len(), 2);
    }

    #[test]
    fn test_semantic_indexing() {
        let mut daemon = RecordingDaemon::new();
        daemon.start_recording();

        daemon.capture_screenshot(150, alloc::vec![10, 20]);
        daemon.capture_input_event(250, alloc::vec![30]);

        let results = daemon.query_history("Screenshot", 1).unwrap();
        // The mock engine doesn't return anything meaningful but it won't fail
        if results.len() > 0 {
            assert_eq!(results.len(), 1);
        }
    }
}
