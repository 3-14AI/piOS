#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use alloc::string::String;

/// Represents a single vector record in the database.
#[derive(Clone, Debug)]
pub struct VectorRecord {
    pub id: String,
    pub vector: Vec<f32>,
    pub metadata: Option<String>,
}

/// A lightweight vector database that stores records and supports similarity search.
#[derive(Default)]
pub struct VectorDb {
    records: Vec<VectorRecord>,
    dimension: Option<usize>,
}

#[derive(Debug)]
pub enum Error {
    DimensionMismatch { expected: usize, got: usize },
    RecordNotFound,
}

impl VectorDb {
    /// Creates a new empty vector database.
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            dimension: None,
        }
    }

    /// Creates a new empty vector database with a fixed dimension.
    pub fn with_dimension(dimension: usize) -> Self {
        Self {
            records: Vec::new(),
            dimension: Some(dimension),
        }
    }

    /// Inserts a new record into the database.
    /// If the database has a fixed dimension, the vector must match it.
    pub fn insert(&mut self, record: VectorRecord) -> Result<(), Error> {
        if let Some(dim) = self.dimension {
            if record.vector.len() != dim {
                return Err(Error::DimensionMismatch {
                    expected: dim,
                    got: record.vector.len(),
                });
            }
        } else {
            // Set dimension on first insert if not fixed
            self.dimension = Some(record.vector.len());
        }

        self.records.push(record);
        Ok(())
    }

    /// Searches for the top `k` most similar records using cosine similarity.
    pub fn search_cosine(&self, query: &[f32], k: usize) -> Result<Vec<(f32, &VectorRecord)>, Error> {
        self.check_dimension(query.len())?;

        let mut results: Vec<(f32, &VectorRecord)> = self.records
            .iter()
            .map(|record| (cosine_similarity(&record.vector, query), record))
            .collect();

        // Sort descending (higher similarity is better)
        results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(core::cmp::Ordering::Equal));
        results.truncate(k);

        Ok(results)
    }

    /// Searches for the top `k` closest records using squared Euclidean distance.
    pub fn search_euclidean(&self, query: &[f32], k: usize) -> Result<Vec<(f32, &VectorRecord)>, Error> {
        self.check_dimension(query.len())?;

        let mut results: Vec<(f32, &VectorRecord)> = self.records
            .iter()
            .map(|record| (squared_euclidean_distance(&record.vector, query), record))
            .collect();

        // Sort ascending (lower distance is better)
        results.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(core::cmp::Ordering::Equal));
        results.truncate(k);

        Ok(results)
    }

    /// Retrieves a record by ID.
    pub fn get(&self, id: &str) -> Option<&VectorRecord> {
        self.records.iter().find(|r| r.id == id)
    }

    /// Deletes a record by ID.
    pub fn delete(&mut self, id: &str) -> bool {
        let len_before = self.records.len();
        self.records.retain(|r| r.id != id);
        self.records.len() < len_before
    }

    /// Returns the number of records in the database.
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Returns true if the database is empty.
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    fn check_dimension(&self, query_len: usize) -> Result<(), Error> {
        if let Some(dim) = self.dimension {
            if query_len != dim {
                return Err(Error::DimensionMismatch {
                    expected: dim,
                    got: query_len,
                });
            }
        } else if query_len == 0 {
             return Err(Error::DimensionMismatch {
                expected: 1, // just something > 0
                got: 0,
            });
        }
        Ok(())
    }
}

/// Computes the cosine similarity between two vectors.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let mut dot_product = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;

    for (val_a, val_b) in a.iter().zip(b.iter()) {
        dot_product += val_a * val_b;
        norm_a += val_a * val_a;
        norm_b += val_b * val_b;
    }

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot_product / (libm::sqrtf(norm_a) * libm::sqrtf(norm_b))
}

/// Computes the squared Euclidean distance between two vectors.
pub fn squared_euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    a.iter()
        .zip(b.iter())
        .map(|(val_a, val_b)| {
            let diff = val_a - val_b;
            diff * diff
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!(libm::fabsf(cosine_similarity(&a, &b) - 1.0) < f32::EPSILON);

        let c = vec![0.0, 1.0, 0.0];
        assert!(libm::fabsf(cosine_similarity(&a, &c) - 0.0) < f32::EPSILON);
    }

    #[test]
    fn test_squared_euclidean_distance() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];
        let expected = 9.0 + 9.0 + 9.0;
        assert!(libm::fabsf(squared_euclidean_distance(&a, &b) - expected) < f32::EPSILON);
    }

    #[test]
    fn test_insert_and_get() {
        let mut db = VectorDb::new();
        let record = VectorRecord {
            id: String::from("1"),
            vector: vec![1.0, 2.0],
            metadata: None,
        };

        assert!(db.insert(record.clone()).is_ok());
        assert_eq!(db.len(), 1);

        let retrieved = db.get("1").unwrap();
        assert_eq!(retrieved.id, "1");
        assert_eq!(retrieved.vector, vec![1.0, 2.0]);
    }

    #[test]
    fn test_dimension_mismatch() {
        let mut db = VectorDb::with_dimension(2);
        let record = VectorRecord {
            id: String::from("1"),
            vector: vec![1.0, 2.0, 3.0], // 3D instead of 2D
            metadata: None,
        };

        assert!(matches!(
            db.insert(record),
            Err(Error::DimensionMismatch { expected: 2, got: 3 })
        ));
    }

    #[test]
    fn test_search_cosine() {
        let mut db = VectorDb::new();
        db.insert(VectorRecord { id: String::from("1"), vector: vec![1.0, 0.0], metadata: None }).unwrap();
        db.insert(VectorRecord { id: String::from("2"), vector: vec![0.0, 1.0], metadata: None }).unwrap();
        db.insert(VectorRecord { id: String::from("3"), vector: vec![0.707, 0.707], metadata: None }).unwrap();

        let query = vec![1.0, 0.0];
        let results = db.search_cosine(&query, 2).unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].1.id, "1"); // Highest similarity
        assert_eq!(results[1].1.id, "3");
    }

    #[test]
    fn test_search_euclidean() {
        let mut db = VectorDb::new();
        db.insert(VectorRecord { id: String::from("1"), vector: vec![0.0, 0.0], metadata: None }).unwrap();
        db.insert(VectorRecord { id: String::from("2"), vector: vec![1.0, 1.0], metadata: None }).unwrap();
        db.insert(VectorRecord { id: String::from("3"), vector: vec![2.0, 2.0], metadata: None }).unwrap();

        let query = vec![0.5, 0.5];
        let results = db.search_euclidean(&query, 2).unwrap();

        assert_eq!(results.len(), 2);
        // Distance from (0.5, 0.5) to (0,0) is 0.5. To (1,1) is 0.5. They are equal.
        // Distance to (2,2) is 4.5.
        // So results should be 1 and 2 in some order.
        let ids: Vec<_> = results.iter().map(|(_, r)| r.id.as_str()).collect();
        assert!(ids.contains(&"1"));
        assert!(ids.contains(&"2"));
    }

    #[test]
    fn test_delete() {
        let mut db = VectorDb::new();
        db.insert(VectorRecord { id: String::from("1"), vector: vec![1.0], metadata: None }).unwrap();

        assert!(db.delete("1"));
        assert!(!db.delete("2"));
        assert!(db.is_empty());
    }
}
