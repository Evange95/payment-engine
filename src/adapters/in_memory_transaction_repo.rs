use crate::domain::transaction::Transaction;
use crate::ports::TransactionRepository;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct InMemoryTransactionRepo {
    transactions: HashMap<u32, Transaction>,
}

impl InMemoryTransactionRepo {
    pub fn new() -> Self {
        Self {
            transactions: HashMap::new(),
        }
    }

}

impl TransactionRepository for InMemoryTransactionRepo {
    fn find_by_tx_id(&self, tx_id: u32) -> Option<Transaction> {
        self.transactions.get(&tx_id).cloned()
    }

    fn save(&mut self, transaction: Transaction) {
        self.transactions.insert(transaction.tx, transaction);
    }
}

impl TransactionRepository for Rc<RefCell<InMemoryTransactionRepo>> {
    fn find_by_tx_id(&self, tx_id: u32) -> Option<Transaction> {
        self.borrow().find_by_tx_id(tx_id)
    }

    fn save(&mut self, transaction: Transaction) {
        self.borrow_mut().save(transaction);
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::transaction::{Transaction, TransactionType};
    use crate::ports::TransactionRepository;

    #[test]
    fn rc_refcell_implements_transaction_repository() {
        use std::cell::RefCell;
        use std::rc::Rc;

        let repo = Rc::new(RefCell::new(super::InMemoryTransactionRepo::new()));

        let mut repo_clone = repo.clone();
        repo_clone.save(Transaction {
            tx_type: TransactionType::Deposit,
            client: 1,
            tx: 42,
            amount: Some("10.0".parse().unwrap()),
        });

        let tx = repo.find_by_tx_id(42).unwrap();
        assert_eq!(tx.client, 1);
        assert_eq!(tx.tx, 42);
    }
}
