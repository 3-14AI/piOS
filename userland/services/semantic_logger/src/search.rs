#![no_std]
#![allow(unused)]

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

pub struct SearchResult {
    pub content: String,
    pub relevance: f32,
}

pub struct SemanticSearchManager {
    // Mock index
    enabled: bool,
}

impl Default for SemanticSearchManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SemanticSearchManager {
    pub fn new() -> Self {
        Self { enabled: true }
    }

    pub fn search(&self, _query: &str) -> Vec<SearchResult> {
        // Mock search implementation
        alloc::vec![SearchResult {
            content: String::from("Mock log entry"),
            relevance: 0.95,
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_search() {
        let manager = SemanticSearchManager::new();
        let results = manager.search("error");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "Mock log entry");
    }
}
