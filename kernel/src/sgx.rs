#![allow(dead_code)]

use alloc::vec::Vec;

pub struct SgxEnclave {
    pub id: u64,
    pub memory: Vec<u8>,
    pub is_running: bool,
}

impl SgxEnclave {
    pub fn new(id: u64, size: usize) -> Self {
        Self {
            id,
            memory: alloc::vec![0; size],
            is_running: false,
        }
    }

    pub fn load_code(&mut self, code: &[u8]) -> bool {
        if code.len() > self.memory.len() {
            return false;
        }
        self.memory[..code.len()].copy_from_slice(code);
        true
    }

    pub fn enter(&mut self) -> bool {
        // Mocking SEV/SGX enter
        self.is_running = true;
        true
    }
}

pub struct SgxManager {
    enclaves: Vec<SgxEnclave>,
    next_id: u64,
}

impl Default for SgxManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SgxManager {
    pub fn new() -> Self {
        Self {
            enclaves: Vec::new(),
            next_id: 1,
        }
    }

    pub fn create_enclave(&mut self, size: usize) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.enclaves.push(SgxEnclave::new(id, size));
        id
    }

    pub fn get_enclave_mut(&mut self, id: u64) -> Option<&mut SgxEnclave> {
        self.enclaves.iter_mut().find(|e| e.id == id)
    }
}
