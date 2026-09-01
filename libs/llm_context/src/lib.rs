#![no_std]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use vector_db::{MultiUserVectorDb, VectorDb, VectorRecord};

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
                        if !self.active_pages.iter().any(|p| p.id == page.id) && self.add_page(page)
                        {
                            loaded_count += 1;
                        }
                    }
                }
            }
        }

        Ok(loaded_count)
    }
}

/// A multi-user LLM context manager that manages separate active windows and vector DB spaces for different users.
pub struct MultiUserLlmContextManager {
    /// Separate active pages for each user.
    active_pages: alloc::collections::BTreeMap<u32, Vec<ContextPage>>,
    /// The maximum number of tokens allowed in the active window per user.
    max_tokens: usize,
    /// The current total tokens in the active window per user.
    current_tokens: alloc::collections::BTreeMap<u32, usize>,
    /// The multi-user vector database.
    pub vector_db: MultiUserVectorDb,
}

impl Default for MultiUserLlmContextManager {
    fn default() -> Self {
        Self::new(8192) // default token limit
    }
}

impl MultiUserLlmContextManager {
    /// Creates a new MultiUserLlmContextManager with the given token limit per user.
    pub fn new(max_tokens: usize) -> Self {
        Self {
            active_pages: alloc::collections::BTreeMap::new(),
            max_tokens,
            current_tokens: alloc::collections::BTreeMap::new(),
            vector_db: MultiUserVectorDb::new(),
        }
    }

    /// Adds a record to a user's vector database space.
    pub fn store_in_db(&mut self, uid: u32, record: VectorRecord) -> Result<(), vector_db::Error> {
        let db = self.vector_db.get_or_create_db(uid);
        db.insert(record)
    }

    /// Helper to get or create the active pages and current tokens for a user
    fn get_or_create_user_state(&mut self, uid: u32) -> (&mut Vec<ContextPage>, &mut usize) {
        let pages = self.active_pages.entry(uid).or_insert_with(Vec::new);
        let tokens = self.current_tokens.entry(uid).or_insert(0);
        (pages, tokens)
    }

    /// Tries to add a page directly to a user's active context window.
    pub fn add_page(&mut self, uid: u32, page: ContextPage) -> bool {
        if page.token_count > self.max_tokens {
            return false;
        }

        let max_tokens = self.max_tokens;

        let (pages, current_tokens) = self.get_or_create_user_state(uid);

        while *current_tokens + page.token_count > max_tokens {
            if pages.is_empty() {
                break;
            }
            let removed = pages.remove(0);
            *current_tokens -= removed.token_count;
        }

        *current_tokens += page.token_count;
        pages.push(page);
        true
    }

    /// Evicts the oldest page from a user's active context window.
    pub fn evict_oldest(&mut self, uid: u32) -> Option<ContextPage> {
        if let Some(pages) = self.active_pages.get_mut(&uid) {
            if pages.is_empty() {
                return None;
            }
            let removed = pages.remove(0);
            if let Some(tokens) = self.current_tokens.get_mut(&uid) {
                *tokens -= removed.token_count;
            }
            return Some(removed);
        }
        None
    }

    /// Evicts all pages from a user's active context window.
    pub fn clear_context(&mut self, uid: u32) {
        if let Some(pages) = self.active_pages.get_mut(&uid) {
            pages.clear();
        }
        if let Some(tokens) = self.current_tokens.get_mut(&uid) {
            *tokens = 0;
        }
    }

    /// Retrieves active pages for a user.
    pub fn get_active_pages(&self, uid: u32) -> Option<&Vec<ContextPage>> {
        self.active_pages.get(&uid)
    }

    /// Retrieves current token count for a user.
    pub fn get_current_tokens(&self, uid: u32) -> usize {
        self.current_tokens.get(&uid).copied().unwrap_or(0)
    }

    /// Searches the vector database for relevant context pages for a specific user and loads them into their active window.
    pub fn load_relevant_context(
        &mut self,
        uid: u32,
        query: &[f32],
        k: usize,
    ) -> Result<usize, vector_db::Error> {
        // Collect references and clone them to release the immutable borrow
        let results = {
            let db = self.vector_db.get_or_create_db(uid);
            db.search_cosine(query, k)?
                .into_iter()
                .map(|(score, rec)| (score, rec.clone()))
                .collect::<Vec<_>>()
        };

        let mut loaded_count = 0;

        for (_score, record) in results {
            if let Some(meta) = &record.metadata {
                if let Some((tokens_str, content)) = meta.split_once('|') {
                    if let Ok(token_count) = tokens_str.parse::<usize>() {
                        let page = ContextPage {
                            id: record.id,
                            content: String::from(content),
                            token_count,
                        };

                        let (pages, _current_tokens) = self.get_or_create_user_state(uid);

                        if !pages.iter().any(|p| p.id == page.id) {
                            if self.add_page(uid, page) {
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

    #[test]
    fn test_multi_user_manager_init() {
        let manager = MultiUserLlmContextManager::new(100);
        assert_eq!(manager.max_tokens, 100);

        let default_manager = MultiUserLlmContextManager::default();
        assert_eq!(default_manager.max_tokens, 8192);
    }

    #[test]
    fn test_multi_user_add_page_and_eviction() {
        let mut manager = MultiUserLlmContextManager::new(100);
        let page1 = ContextPage {
            id: "1".to_string(),
            content: "Hello User 1".to_string(),
            token_count: 50,
        };
        let page2 = ContextPage {
            id: "2".to_string(),
            content: "World User 1".to_string(),
            token_count: 60,
        };
        let page3 = ContextPage {
            id: "3".to_string(),
            content: "Hello User 2".to_string(),
            token_count: 80,
        };

        assert!(manager.add_page(1000, page1));
        assert_eq!(manager.get_current_tokens(1000), 50);

        assert!(manager.add_page(1001, page3));
        assert_eq!(manager.get_current_tokens(1001), 80);

        // Adding page2 for User 1000 should evict page1
        assert!(manager.add_page(1000, page2));
        assert_eq!(manager.get_current_tokens(1000), 60);
        assert_eq!(manager.get_active_pages(1000).unwrap().len(), 1);
        assert_eq!(manager.get_active_pages(1000).unwrap()[0].id, "2");

        // User 1001 should be unaffected
        assert_eq!(manager.get_current_tokens(1001), 80);
        assert_eq!(manager.get_active_pages(1001).unwrap().len(), 1);
    }

    #[test]
    fn test_multi_user_clear_context() {
        let mut manager = MultiUserLlmContextManager::new(100);
        manager.add_page(
            1000,
            ContextPage {
                id: "1".to_string(),
                content: "Hello".to_string(),
                token_count: 50,
            },
        );
        manager.add_page(
            1001,
            ContextPage {
                id: "2".to_string(),
                content: "World".to_string(),
                token_count: 50,
            },
        );

        manager.clear_context(1000);
        assert_eq!(manager.get_current_tokens(1000), 0);
        assert!(manager.get_active_pages(1000).unwrap().is_empty());

        assert_eq!(manager.get_current_tokens(1001), 50);
        assert_eq!(manager.get_active_pages(1001).unwrap().len(), 1);
    }

    #[test]
    fn test_multi_user_load_relevant_context() {
        let mut manager = MultiUserLlmContextManager::new(100);

        manager
            .store_in_db(
                1000,
                VectorRecord {
                    id: "1".to_string(),
                    vector: vec![1.0, 0.0],
                    metadata: Some("30|Relevant Content User 1000".to_string()),
                },
            )
            .unwrap();

        manager
            .store_in_db(
                1001,
                VectorRecord {
                    id: "2".to_string(),
                    vector: vec![1.0, 0.0],
                    metadata: Some("40|Relevant Content User 1001".to_string()),
                },
            )
            .unwrap();

        let query = vec![1.0, 0.0];

        let loaded = manager.load_relevant_context(1000, &query, 1).unwrap();
        assert_eq!(loaded, 1);
        assert_eq!(manager.get_current_tokens(1000), 30);
        assert_eq!(
            manager.get_active_pages(1000).unwrap()[0].content,
            "Relevant Content User 1000"
        );
        assert_eq!(manager.get_current_tokens(1001), 0);

        let loaded2 = manager.load_relevant_context(1001, &query, 1).unwrap();
        assert_eq!(loaded2, 1);
        assert_eq!(manager.get_current_tokens(1001), 40);
        assert_eq!(
            manager.get_active_pages(1001).unwrap()[0].content,
            "Relevant Content User 1001"
        );
    }
}
