use crate::LlmContextManager;
use alloc::format;
use alloc::string::String;

pub const VERUS_SYSTEM_PROMPT: &str = "\
You are an expert Rust programmer specialized in writing formally verified code using Verus.
Follow these guidelines strictly:
1. Write `#![no_std]` compatible code whenever possible, as the target is often a kernel or WASM environment.
2. Use Verus features (`vstd`) for formal verification, ensuring you include necessary `requires` and `ensures` clauses.
3. Keep invariants clear and concise.
4. If `#![no_std]` is used, rely on `alloc` for heap allocations (`alloc::vec::Vec`, `alloc::string::String`).
5. Ensure safety and absence of undefined behavior.
6. When performing bounds checking, rewrite additions as subtractions to prevent verifier failures related to potential arithmetic overflow.
7. Use `Matches` or standard Rust `is_some()`/`is_none()` where appropriate instead of deprecated `#[is_variant]`.
";

pub fn build_verus_rag_prompt(
    manager: &mut LlmContextManager,
    query: &str,
    query_embedding: &[f32],
    k: usize,
) -> Result<String, vector_db::Error> {
    manager.load_relevant_context(query_embedding, k)?;

    let mut context_str = String::new();
    for page in &manager.active_pages {
        context_str.push_str(&format!("--- Context Document {} ---\n{}\n\n", page.id, page.content));
    }

    let final_prompt = format!(
        "{}\n\nRelevant Context:\n{}\n\nUser Query:\n{}",
        VERUS_SYSTEM_PROMPT, context_str, query
    );

    Ok(final_prompt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;
    use alloc::vec;
    use vector_db::VectorRecord;

    #[test]
    fn test_build_verus_rag_prompt() {
        let mut manager = LlmContextManager::new(100);

        // Store some mock context in the vector DB
        let record = VectorRecord {
            id: "verus_doc_1".to_string(),
            vector: vec![1.0, 0.0],
            metadata: Some("20|Requires clauses must be placed before ensures clauses.".to_string()),
        };
        manager.store_in_db(record).unwrap();

        // Query the context to build the prompt
        let query = "How do I specify a precondition?";
        let query_embedding = vec![1.0, 0.0];

        let prompt = build_verus_rag_prompt(&mut manager, query, &query_embedding, 1).unwrap();

        assert!(prompt.contains(VERUS_SYSTEM_PROMPT));
        assert!(prompt.contains("--- Context Document verus_doc_1 ---"));
        assert!(prompt.contains("Requires clauses must be placed before ensures clauses."));
        assert!(prompt.contains("User Query:"));
        assert!(prompt.contains(query));
    }
}
