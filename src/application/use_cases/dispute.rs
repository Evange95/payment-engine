use crate::ports::{AccountRepository, TransactionRepository};

pub struct DisputeUseCase<A: AccountRepository, T: TransactionRepository> {
    account_repo: A,
    tx_repo: T,
}

impl<A: AccountRepository, T: TransactionRepository> DisputeUseCase<A, T> {
    pub fn new(account_repo: A, tx_repo: T) -> Self {
        Self {
            account_repo,
            tx_repo,
        }
    }

    pub fn execute(&mut self, client_id: u16, tx_id: u32) {
        let tx = match self.tx_repo.find_by_tx_id(tx_id) {
            Some(tx) => tx,
            None => return,
        };

        let mut account = match self.account_repo.find_by_client_id(client_id) {
            Some(account) => account,
            None => return,
        };

        let disputed_amount = match tx.amount {
            Some(a) => a,
            None => return,
        };

        account.available = account.available - disputed_amount;
        account.held = account.held + disputed_amount;
        self.account_repo.save(account);
    }

    pub fn account_repo(&self) -> &A {
        &self.account_repo
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::account::Account;
    use crate::domain::amount::Amount;
    use crate::domain::transaction::{Transaction, TransactionType};
    use crate::ports::{AccountRepository, TransactionRepository};
    use std::collections::HashMap;

    struct InMemoryAccountRepo {
        accounts: HashMap<u16, Account>,
    }

    impl InMemoryAccountRepo {
        fn new() -> Self {
            Self {
                accounts: HashMap::new(),
            }
        }

        fn get(&self, client_id: u16) -> Option<&Account> {
            self.accounts.get(&client_id)
        }
    }

    impl AccountRepository for InMemoryAccountRepo {
        fn find_by_client_id(&self, client_id: u16) -> Option<Account> {
            self.accounts.get(&client_id).cloned()
        }

        fn save(&mut self, account: Account) {
            self.accounts.insert(account.client, account);
        }
    }

    struct InMemoryTransactionRepo {
        transactions: HashMap<u32, Transaction>,
    }

    impl InMemoryTransactionRepo {
        fn new() -> Self {
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

    fn amount(s: &str) -> Amount {
        s.parse().unwrap()
    }

    #[test]
    fn moves_funds_from_available_to_held() {
        let mut account_repo = InMemoryAccountRepo::new();
        account_repo.save(Account {
            client: 1,
            available: amount("100.0"),
            held: Amount::ZERO,
            locked: false,
        });

        let mut tx_repo = InMemoryTransactionRepo::new();
        tx_repo.save(Transaction {
            tx_type: TransactionType::Deposit,
            client: 1,
            tx: 42,
            amount: Some(amount("30.0")),
        });

        let mut use_case = super::DisputeUseCase::new(account_repo, tx_repo);
        use_case.execute(1, 42);

        let account = use_case.account_repo().get(1).unwrap();
        assert_eq!(account.available, amount("70.0"));
        assert_eq!(account.held, amount("30.0"));
        assert_eq!(account.total(), amount("100.0"));
    }

    #[test]
    fn ignores_non_existent_transaction() {
        let mut account_repo = InMemoryAccountRepo::new();
        account_repo.save(Account {
            client: 1,
            available: amount("100.0"),
            held: Amount::ZERO,
            locked: false,
        });
        let tx_repo = InMemoryTransactionRepo::new();

        let mut use_case = super::DisputeUseCase::new(account_repo, tx_repo);
        use_case.execute(1, 999);

        let account = use_case.account_repo().get(1).unwrap();
        assert_eq!(account.available, amount("100.0"));
        assert_eq!(account.held, Amount::ZERO);
    }

    #[test]
    fn ignores_non_existent_account() {
        let account_repo = InMemoryAccountRepo::new();
        let mut tx_repo = InMemoryTransactionRepo::new();
        tx_repo.save(Transaction {
            tx_type: TransactionType::Deposit,
            client: 1,
            tx: 42,
            amount: Some(amount("30.0")),
        });

        let mut use_case = super::DisputeUseCase::new(account_repo, tx_repo);
        use_case.execute(1, 42);
        // no panic = pass
    }

    #[test]
    fn total_funds_unchanged_with_existing_held() {
        let mut account_repo = InMemoryAccountRepo::new();
        account_repo.save(Account {
            client: 1,
            available: amount("200.0"),
            held: amount("50.0"),
            locked: false,
        });
        let mut tx_repo = InMemoryTransactionRepo::new();
        tx_repo.save(Transaction {
            tx_type: TransactionType::Deposit,
            client: 1,
            tx: 7,
            amount: Some(amount("25.0")),
        });

        let mut use_case = super::DisputeUseCase::new(account_repo, tx_repo);
        use_case.execute(1, 7);

        let account = use_case.account_repo().get(1).unwrap();
        assert_eq!(account.available, amount("175.0"));
        assert_eq!(account.held, amount("75.0"));
        assert_eq!(account.total(), amount("250.0"));
    }
}
