use crate::ports::{AccountRepository, DisputeRepository, TransactionRepository};

pub struct ResolveUseCase<A: AccountRepository, T: TransactionRepository, D: DisputeRepository> {
    account_repo: A,
    tx_repo: T,
    dispute_repo: D,
}

impl<A: AccountRepository, T: TransactionRepository, D: DisputeRepository>
    ResolveUseCase<A, T, D>
{
    pub fn new(account_repo: A, tx_repo: T, dispute_repo: D) -> Self {
        Self {
            account_repo,
            tx_repo,
            dispute_repo,
        }
    }

    pub fn execute(&mut self, client_id: u16, tx_id: u32) {
        if !self.dispute_repo.is_disputed(tx_id) {
            return;
        }

        let tx = match self.tx_repo.find_by_tx_id(tx_id) {
            Some(tx) => tx,
            None => return,
        };

        let mut account = match self.account_repo.find_by_client_id(client_id) {
            Some(account) => account,
            None => return,
        };

        let amount = match tx.amount {
            Some(a) => a,
            None => return,
        };

        account.held = account.held - amount;
        account.available = account.available + amount;
        self.account_repo.save(account);
        self.dispute_repo.remove_dispute(tx_id);
    }

    pub fn account_repo(&self) -> &A {
        &self.account_repo
    }

    pub fn dispute_repo(&self) -> &D {
        &self.dispute_repo
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::account::Account;
    use crate::domain::amount::Amount;
    use crate::domain::transaction::{Transaction, TransactionType};
    use crate::ports::{AccountRepository, DisputeRepository, TransactionRepository};
    use std::collections::HashMap;
    use std::collections::HashSet;

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

    struct InMemoryDisputeRepo {
        disputed: HashSet<u32>,
    }

    impl InMemoryDisputeRepo {
        fn new() -> Self {
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

    fn amount(s: &str) -> Amount {
        s.parse().unwrap()
    }

    #[test]
    fn releases_held_funds_to_available() {
        let mut account_repo = InMemoryAccountRepo::new();
        account_repo.save(Account {
            client: 1,
            available: amount("70.0"),
            held: amount("30.0"),
            locked: false,
        });

        let mut tx_repo = InMemoryTransactionRepo::new();
        tx_repo.save(Transaction {
            tx_type: TransactionType::Deposit,
            client: 1,
            tx: 42,
            amount: Some(amount("30.0")),
        });

        let mut dispute_repo = InMemoryDisputeRepo::new();
        dispute_repo.mark_disputed(42);

        let mut use_case = super::ResolveUseCase::new(account_repo, tx_repo, dispute_repo);
        use_case.execute(1, 42);

        let account = use_case.account_repo().get(1).unwrap();
        assert_eq!(account.available, amount("100.0"));
        assert_eq!(account.held, Amount::ZERO);
        assert_eq!(account.total(), amount("100.0"));
    }
}
