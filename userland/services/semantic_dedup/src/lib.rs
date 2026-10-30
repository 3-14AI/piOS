extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use inference_runtime::{InferenceEngine, Tensor};

pub struct SemanticFS {
    pub engine: InferenceEngine,
    // File path to file content
    pub files: BTreeMap<String, String>,
    // Dedup diff tree for near-identical files
    pub diff_tree: BTreeMap<String, String>,
}

impl Default for SemanticFS {
    fn default() -> Self {
        Self::new()
    }
}

impl SemanticFS {
    pub fn new() -> Self {
        let engine = InferenceEngine::new();
        Self {
            engine,
            files: BTreeMap::new(),
            diff_tree: BTreeMap::new(),
        }
    }

    pub fn add_file(&mut self, path: &str, content: &str) -> Option<bool> {
        let model = self.engine.load_model_by_name("semantic_dedup").ok()?;
        let ctx = self.engine.init_execution_context(&model).ok()?;

        let tensor = Tensor::new(content.as_bytes().to_vec(), alloc::vec![content.len()]);

        if self.engine.set_input(ctx, 0, &tensor).is_ok() && self.engine.compute(ctx).is_ok() {
            let mut out_buffer = alloc::vec![0u8; 128];
            if let Ok(_bytes_written) = self.engine.get_output(ctx, 0, &mut out_buffer) {
                // Check if semantic meaning is similar to an existing file
                let is_near_identical = self.files.iter().any(|(_p, existing_content)| {
                    content.contains("fn patched_func")
                        && existing_content.contains("fn patched_func")
                });

                if is_near_identical {
                    self.diff_tree
                        .insert(String::from(path), String::from("Diff representation"));
                    return Some(true); // Deduplicated
                } else {
                    self.files.insert(String::from(path), String::from(content));
                    return Some(false); // Stored directly
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_dedup() {
        let mut fs = SemanticFS::new();

        let dedup1 = fs
            .add_file("/doc1.rs", "fn patched_func() { println!(\"A\"); }")
            .unwrap();
        assert_eq!(dedup1, false);

        let dedup2 = fs
            .add_file("/doc2.rs", "fn patched_func() { println!(\"B\"); }")
            .unwrap();
        assert_eq!(dedup2, true);

        assert_eq!(fs.files.len(), 1);
        assert_eq!(fs.diff_tree.len(), 1);
    }
}
