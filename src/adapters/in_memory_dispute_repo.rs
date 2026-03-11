use crate::ports::DisputeRepository;
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

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

impl DisputeRepository for Rc<RefCell<InMemoryDisputeRepo>> {
    fn is_disputed(&self, tx_id: u32) -> bool {
        self.borrow().is_disputed(tx_id)
    }

    fn mark_disputed(&mut self, tx_id: u32) {
        self.borrow_mut().mark_disputed(tx_id);
    }

    fn remove_dispute(&mut self, tx_id: u32) {
        self.borrow_mut().remove_dispute(tx_id);
    }
}

#[cfg(test)]
mod tests {
    use crate::ports::DisputeRepository;

    #[test]
    fn rc_refcell_implements_dispute_repository() {
        use std::cell::RefCell;
        use std::rc::Rc;

        let repo = Rc::new(RefCell::new(super::InMemoryDisputeRepo::new()));

        let mut repo_clone = repo.clone();
        repo_clone.mark_disputed(42);

        assert!(repo.is_disputed(42));

        let mut repo_clone2 = repo.clone();
        repo_clone2.remove_dispute(42);

        assert!(!repo.is_disputed(42));
    }
}
