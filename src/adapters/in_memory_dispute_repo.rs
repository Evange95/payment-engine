use crate::ports::DisputeRepository;
use std::collections::HashSet;

pub struct InMemoryDisputeRepo {
    disputed: HashSet<u32>,
}

impl InMemoryDisputeRepo {
    pub fn new() -> Self {
        Self {
            disputed: HashSet::new(),
        }
    }
}

impl DisputeRepository for InMemoryDisputeRepo {
    fn is_disputed(&self, tx_id: u32) -> bool {
        self.disputed.contains(&tx_id)
    }

    fn mark_disputed(&mut self, tx_id: u32) {
        self.disputed.insert(tx_id);
    }

    fn remove_dispute(&mut self, tx_id: u32) {
        self.disputed.remove(&tx_id);
    }
}
