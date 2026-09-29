use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};

pub enum JournalEntry {
    AddFile(u64, String),
    RemoveFile(u64),
    UpdateFile(u64, String),
}

pub struct SemanticJournal {
    entries: Vec<JournalEntry>,
    tx_id: AtomicUsize,
    is_recovering: bool,
}

impl Default for SemanticJournal {
    fn default() -> Self {
        Self::new()
    }
}

impl SemanticJournal {
    pub fn new() -> Self {
        Self {
            entries: alloc::vec![],
            tx_id: AtomicUsize::new(0),
            is_recovering: false,
        }
    }

    pub fn begin_tx(&mut self) -> usize {
        self.tx_id.fetch_add(1, Ordering::SeqCst)
    }

    pub fn commit_tx(&mut self, _id: usize) {
        // Mock fsync
    }

    pub fn log_add_file(&mut self, inode: u64, content: &str) {
        if !self.is_recovering {
            self.entries
                .push(JournalEntry::AddFile(inode, String::from(content)));
        }
    }

    pub fn log_remove_file(&mut self, inode: u64) {
        if !self.is_recovering {
            self.entries.push(JournalEntry::RemoveFile(inode));
        }
    }

    pub fn log_update_file(&mut self, inode: u64, content: &str) {
        if !self.is_recovering {
            self.entries
                .push(JournalEntry::UpdateFile(inode, String::from(content)));
        }
    }

    pub fn recover<F>(&mut self, mut apply: F) -> Result<(), &'static str>
    where
        F: FnMut(&JournalEntry),
    {
        self.is_recovering = true;
        for entry in &self.entries {
            apply(entry);
        }
        self.is_recovering = false;
        Ok(())
    }
}
