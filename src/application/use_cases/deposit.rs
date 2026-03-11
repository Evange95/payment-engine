use crate::domain::account::Account;
use crate::domain::amount::Amount;
use crate::domain::transaction::{Transaction, TransactionType};
use crate::ports::{AccountRepository, TransactionRepository};

pub struct DepositUseCase<A: AccountRepository, T: TransactionRepository> {
    account_repo: A,
    tx_repo: T,
}

impl<A: AccountRepository, T: TransactionRepository> DepositUseCase<A, T> {
    pub fn new(account_repo: A, tx_repo: T) -> Self {
        Self {
            account_repo,
            tx_repo,
        }
    }

    pub fn execute(&mut self, client_id: u16, tx: u32, amount: Amount) {
        let mut account = self
            .account_repo
            .find_by_client_id(client_id)
            .unwrap_or_else(|| Account::new(client_id));

        account.available = account.available + amount;
        self.account_repo.save(account);

        self.tx_repo.save(Transaction {
            tx_type: TransactionType::Deposit,
            client: client_id,
            tx,
            amount: Some(amount),
        });
    }

    pub fn repo(&self) -> &A {
        &self.account_repo
    }

    pub fn tx_repo(&self) -> &T {
        &self.tx_repo
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

    fn amount(s: &str) -> Amount {
        s.parse().unwrap()
    }

    #[test]
    fn increases_available_and_total() {
        let mut repo = InMemoryAccountRepo::new();
        repo.save(Account {
            client: 1,
            available: amount("30.0"),
            held: amount("10.0"),
            locked: false,
        });
        let mut use_case = super::DepositUseCase::new(repo, InMemoryTransactionRepo::new());

        use_case.execute(1, 1, amount("20.0"));

        let account = use_case.repo().get(1).unwrap();
        assert_eq!(account.available, amount("50.0"));
        assert_eq!(account.total(), amount("60.0"));
    }

    #[test]
    fn creates_account_on_first_deposit() {
        let repo = InMemoryAccountRepo::new();
        let mut use_case = super::DepositUseCase::new(repo, InMemoryTransactionRepo::new());

        use_case.execute(1, 1, amount("10.0"));

        let account = use_case.repo().get(1).expect("account should exist");
        assert_eq!(account.available, amount("10.0"));
        assert_eq!(account.held, Amount::ZERO);
        assert!(!account.locked);
    }

    #[test]
    fn adds_to_existing_available_balance() {
        let mut repo = InMemoryAccountRepo::new();
        repo.save(Account {
            client: 1,
            available: amount("50.0"),
            held: Amount::ZERO,
            locked: false,
        });
        let mut use_case = super::DepositUseCase::new(repo, InMemoryTransactionRepo::new());

        use_case.execute(1, 1, amount("25.5"));

        let account = use_case.repo().get(1).unwrap();
        assert_eq!(account.available, amount("75.5"));
    }

    #[test]
    fn deposits_to_separate_clients_independently() {
        let repo = InMemoryAccountRepo::new();
        let mut use_case = super::DepositUseCase::new(repo, InMemoryTransactionRepo::new());

        use_case.execute(1, 1, amount("100.0"));
        use_case.execute(2, 2, amount("200.0"));

        assert_eq!(use_case.repo().get(1).unwrap().available, amount("100.0"));
        assert_eq!(use_case.repo().get(2).unwrap().available, amount("200.0"));
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

        fn get(&self, tx_id: u32) -> Option<&Transaction> {
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

    #[test]
    fn saves_transaction_record() {
        let account_repo = InMemoryAccountRepo::new();
        let tx_repo = InMemoryTransactionRepo::new();
        let mut use_case = super::DepositUseCase::new(account_repo, tx_repo);

        use_case.execute(1, 42, amount("10.0"));

        let tx = use_case.tx_repo().get(42).expect("transaction should be saved");
        assert_eq!(tx.tx_type, TransactionType::Deposit);
        assert_eq!(tx.client, 1);
        assert_eq!(tx.tx, 42);
        assert_eq!(tx.amount, Some(amount("10.0")));
    }
}
