use crate::domain::transaction::Transaction;
use crate::ports::TransactionRepository;
use std::collections::HashMap;

pub struct InMemoryTransactionRepo {
    transactions: HashMap<u32, Transaction>,
}

impl InMemoryTransactionRepo {
    pub fn new() -> Self {
        Self {
            transactions: HashMap::new(),
        }
    }

    pub fn get(&self, tx_id: u32) -> Option<&Transaction> {
        self.transactions.get(&tx_id)
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
