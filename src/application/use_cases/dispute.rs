use crate::domain::account::Account;
use crate::ports::{AccountRepository, DisputeRepository, DisputeTx, TransactionRepository};

pub struct DisputeUseCase<A: AccountRepository, T: TransactionRepository, D: DisputeRepository> {
    account_repo: A,
    tx_repo: T,
    dispute_repo: D,
}

impl<A: AccountRepository, T: TransactionRepository, D: DisputeRepository>
    DisputeUseCase<A, T, D>
{
    pub fn new(account_repo: A, tx_repo: T, dispute_repo: D) -> Self {
        Self {
            account_repo,
            tx_repo,
            dispute_repo,
        }
    }

    pub fn execute(&mut self, client_id: u16, tx_id: u32) -> Option<Account> {
        let tx = self.tx_repo.find_by_tx_id(tx_id)?;
        let mut account = self.account_repo.find_by_client_id(client_id)?;
        let disputed_amount = tx.amount?;

        account.available = account.available - disputed_amount;
        account.held = account.held + disputed_amount;
        self.account_repo.save(account.clone());
        self.dispute_repo.mark_disputed(tx_id);
        Some(account)
    }

    pub fn account_repo(&self) -> &A {
        &self.account_repo
    }

    pub fn dispute_repo(&self) -> &D {
        &self.dispute_repo
    }
}

impl<A: AccountRepository, T: TransactionRepository, D: DisputeRepository> DisputeTx
    for DisputeUseCase<A, T, D>
{
    fn execute(&mut self, client_id: u16, tx_id: u32) -> Option<Account> {
        self.execute(client_id, tx_id)
    }
}

#[cfg(test)]
mod tests {
    use crate::adapters::in_memory_account_repo::InMemoryAccountRepo;
    use crate::adapters::in_memory_dispute_repo::InMemoryDisputeRepo;
    use crate::adapters::in_memory_transaction_repo::InMemoryTransactionRepo;
    use crate::domain::account::Account;
    use crate::domain::amount::Amount;
    use crate::domain::transaction::{Transaction, TransactionType};
    use crate::ports::{AccountRepository, DisputeRepository, TransactionRepository};

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

        let dispute_repo = InMemoryDisputeRepo::new();
        let mut use_case = super::DisputeUseCase::new(account_repo, tx_repo, dispute_repo);
        use_case.execute(1, 42);

        let account = use_case.account_repo().get(1).unwrap();
        assert_eq!(account.available, amount("70.0"));
        assert_eq!(account.held, amount("30.0"));
        assert_eq!(account.total(), amount("100.0"));
    }

    #[test]
    fn marks_transaction_as_disputed() {
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

        let dispute_repo = InMemoryDisputeRepo::new();
        let mut use_case = super::DisputeUseCase::new(account_repo, tx_repo, dispute_repo);
        use_case.execute(1, 42);

        assert!(use_case.dispute_repo().is_disputed(42));
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
        let dispute_repo = InMemoryDisputeRepo::new();

        let mut use_case = super::DisputeUseCase::new(account_repo, tx_repo, dispute_repo);
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
        let dispute_repo = InMemoryDisputeRepo::new();

        let mut use_case = super::DisputeUseCase::new(account_repo, tx_repo, dispute_repo);
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
        let dispute_repo = InMemoryDisputeRepo::new();

        let mut use_case = super::DisputeUseCase::new(account_repo, tx_repo, dispute_repo);
        use_case.execute(1, 7);

        let account = use_case.account_repo().get(1).unwrap();
        assert_eq!(account.available, amount("175.0"));
        assert_eq!(account.held, amount("75.0"));
        assert_eq!(account.total(), amount("250.0"));
    }
}
