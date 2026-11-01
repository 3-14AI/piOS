use alloc::string::String;
use alloc::vec::Vec;

pub struct MultiSwarm {
    pub interconnected_clusters: Vec<String>,
}

impl MultiSwarm {
    pub fn new() -> Self {
        Self {
            interconnected_clusters: Vec::new(),
        }
    }

    pub fn interconnect(&mut self, cluster: &str) {
        self.interconnected_clusters.push(String::from(cluster));
    }
}
