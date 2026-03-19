#![no_std]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use vector_db::{VectorDb, VectorRecord};

/// Represents a single piece of context that can be loaded into the LLM's active window.
#[derive(Clone, Debug)]
pub struct ContextPage {
    pub id: String,
    pub content: String,
    pub token_count: usize,
}

/// The state of the LLM context manager.
pub struct LlmContextManager {
    /// Active pages in the context window.
    pub active_pages: Vec<ContextPage>,
    /// The maximum number of tokens allowed in the active window.
    pub max_tokens: usize,
    /// The current total tokens in the active window.
    pub current_tokens: usize,
    /// The vector database for retrieving relevant context.
    pub vector_db: VectorDb,
}

impl Default for LlmContextManager {
    fn default() -> Self {
        Self::new(8192) // default token limit
    }
}

impl LlmContextManager {
    /// Creates a new LlmContextManager with the given token limit.
    pub fn new(max_tokens: usize) -> Self {
        Self {
            active_pages: Vec::new(),
            max_tokens,
            current_tokens: 0,
            vector_db: VectorDb::new(),
        }
    }

    /// Creates a new manager with a pre-existing vector database.
    pub fn with_db(max_tokens: usize, vector_db: VectorDb) -> Self {
        Self {
            active_pages: Vec::new(),
            max_tokens,
            current_tokens: 0,
            vector_db,
        }
    }

    /// Adds a record to the vector database.
    pub fn store_in_db(&mut self, record: VectorRecord) -> Result<(), vector_db::Error> {
        self.vector_db.insert(record)
    }

    /// Tries to add a page directly to the active context window.
    /// If it exceeds the token limit, it will evict older pages until there's enough room,
    /// or return false if the page itself is larger than the max limit.
    pub fn add_page(&mut self, page: ContextPage) -> bool {
        if page.token_count > self.max_tokens {
            return false;
        }

        while self.current_tokens + page.token_count > self.max_tokens {
            self.evict_oldest();
        }

        self.current_tokens += page.token_count;
        self.active_pages.push(page);
        true
    }

    /// Evicts the oldest page from the active context window.
    pub fn evict_oldest(&mut self) -> Option<ContextPage> {
        if self.active_pages.is_empty() {
            return None;
        }

        // Remove from the front (oldest)
        let removed = self.active_pages.remove(0);
        self.current_tokens -= removed.token_count;
        Some(removed)
    }

    /// Evicts all pages from the active context window.
    pub fn clear_context(&mut self) {
        self.active_pages.clear();
        self.current_tokens = 0;
    }

    /// Searches the vector database for relevant context pages and loads them into the active window.
    /// Uses cosine similarity to find the top `k` records, and loads them if there's space.
    pub fn load_relevant_context(
        &mut self,
        query: &[f32],
        k: usize,
    ) -> Result<usize, vector_db::Error> {
        // Collect references and clone them to release the immutable borrow
        let results: Vec<_> = self
            .vector_db
            .search_cosine(query, k)?
            .into_iter()
            .map(|(score, rec)| (score, rec.clone()))
            .collect();

        let mut loaded_count = 0;

        for (_score, record) in results {
            // Reconstruct a ContextPage from the VectorRecord.
            // In a real scenario, the metadata might contain the text content and token count,
            // or we'd fetch it from a secondary store using the record ID.
            // Here, we simulate by parsing the metadata if it's available.
            if let Some(meta) = &record.metadata {
                // Expecting metadata to be in format "token_count|content" for simplicity in this MVP
                if let Some((tokens_str, content)) = meta.split_once('|') {
                    if let Ok(token_count) = tokens_str.parse::<usize>() {
                        let page = ContextPage {
                            id: record.id,
                            content: String::from(content),
                            token_count,
                        };

                        // Check if we already have it to avoid duplicates
                        if !self.active_pages.iter().any(|p| p.id == page.id) {
                            if self.add_page(page) {
                                loaded_count += 1;
                            }
                        }
                    }
                }
            }
        }

        Ok(loaded_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;
    use alloc::vec;

    #[test]
    fn test_manager_init() {
        let manager = LlmContextManager::new(100);
        assert_eq!(manager.max_tokens, 100);
        assert_eq!(manager.current_tokens, 0);
        assert!(manager.active_pages.is_empty());

        let default_manager = LlmContextManager::default();
        assert_eq!(default_manager.max_tokens, 8192);
    }

    #[test]
    fn test_add_page_success() {
        let mut manager = LlmContextManager::new(100);
        let page = ContextPage {
            id: "1".to_string(),
            content: "Hello".to_string(),
            token_count: 50,
        };

        assert!(manager.add_page(page.clone()));
        assert_eq!(manager.current_tokens, 50);
        assert_eq!(manager.active_pages.len(), 1);
    }

    #[test]
    fn test_add_page_too_large() {
        let mut manager = LlmContextManager::new(100);
        let page = ContextPage {
            id: "1".to_string(),
            content: "Huge".to_string(),
            token_count: 150,
        };

        assert!(!manager.add_page(page));
        assert_eq!(manager.current_tokens, 0);
        assert_eq!(manager.active_pages.len(), 0);
    }

    #[test]
    fn test_eviction() {
        let mut manager = LlmContextManager::new(100);
        let page1 = ContextPage {
            id: "1".to_string(),
            content: "Hello".to_string(),
            token_count: 50,
        };
        let page2 = ContextPage {
            id: "2".to_string(),
            content: "World".to_string(),
            token_count: 60,
        };

        assert!(manager.add_page(page1));
        assert_eq!(manager.current_tokens, 50);

        // Adding page2 should evict page1 since 50 + 60 > 100
        assert!(manager.add_page(page2));
        assert_eq!(manager.current_tokens, 60);
        assert_eq!(manager.active_pages.len(), 1);
        assert_eq!(manager.active_pages[0].id, "2");
    }

    #[test]
    fn test_clear_context() {
        let mut manager = LlmContextManager::new(100);
        manager.add_page(ContextPage {
            id: "1".to_string(),
            content: "Hello".to_string(),
            token_count: 50,
        });

        manager.clear_context();
        assert_eq!(manager.current_tokens, 0);
        assert!(manager.active_pages.is_empty());
    }

    #[test]
    fn test_load_relevant_context() {
        let mut manager = LlmContextManager::new(100);

        // Insert a record into vector db
        let record = VectorRecord {
            id: "1".to_string(),
            vector: vec![1.0, 0.0],
            metadata: Some("30|Relevant Content".to_string()),
        };
        manager.store_in_db(record).unwrap();

        // Search for it
        let query = vec![1.0, 0.0];
        let loaded = manager.load_relevant_context(&query, 1).unwrap();

        assert_eq!(loaded, 1);
        assert_eq!(manager.current_tokens, 30);
        assert_eq!(manager.active_pages.len(), 1);
        assert_eq!(manager.active_pages[0].content, "Relevant Content");

        // Search again, shouldn't load duplicate
        let loaded2 = manager.load_relevant_context(&query, 1).unwrap();
        assert_eq!(loaded2, 0);
    }
}
