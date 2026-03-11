use crate::domain::account::Account;
use crate::ports::{
    AccountRepository, Chargeback, DisputeRepository, TransactionRepository,
};

pub struct ChargebackUseCase<A: AccountRepository, T: TransactionRepository, D: DisputeRepository> {
    account_repo: A,
    tx_repo: T,
    dispute_repo: D,
}

impl<A: AccountRepository, T: TransactionRepository, D: DisputeRepository>
    ChargebackUseCase<A, T, D>
{
    pub fn new(account_repo: A, tx_repo: T, dispute_repo: D) -> Self {
        Self {
            account_repo,
            tx_repo,
            dispute_repo,
        }
    }

    pub fn execute(&mut self, client_id: u16, tx_id: u32) -> Option<Account> {
        if !self.dispute_repo.is_disputed(tx_id) {
            return None;
        }

        let tx = self.tx_repo.find_by_tx_id(tx_id)?;
        let mut account = self.account_repo.find_by_client_id(client_id)?;
        let amount = tx.amount?;

        account.held = account.held - amount;
        account.locked = true;
        self.account_repo.save(account.clone());
        self.dispute_repo.remove_dispute(tx_id);
        Some(account)
    }

    pub fn account_repo(&self) -> &A {
        &self.account_repo
    }

    pub fn dispute_repo(&self) -> &D {
        &self.dispute_repo
    }
}

impl<A: AccountRepository, T: TransactionRepository, D: DisputeRepository> Chargeback
    for ChargebackUseCase<A, T, D>
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
    fn withdraws_held_funds_and_freezes_account() {
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

        let mut use_case = super::ChargebackUseCase::new(account_repo, tx_repo, dispute_repo);
        use_case.execute(1, 42);

        let account = use_case.account_repo().get(1).unwrap();
        assert_eq!(account.available, amount("70.0"));
        assert_eq!(account.held, Amount::ZERO);
        assert_eq!(account.total(), amount("70.0"));
        assert!(account.locked);
    }

    #[test]
    fn ignores_non_disputed_transaction() {
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

        let dispute_repo = InMemoryDisputeRepo::new(); // not disputed

        let mut use_case = super::ChargebackUseCase::new(account_repo, tx_repo, dispute_repo);
        use_case.execute(1, 42);

        let account = use_case.account_repo().get(1).unwrap();
        assert_eq!(account.available, amount("70.0"));
        assert_eq!(account.held, amount("30.0"));
        assert!(!account.locked);
    }

    #[test]
    fn ignores_non_existent_transaction() {
        let mut account_repo = InMemoryAccountRepo::new();
        account_repo.save(Account {
            client: 1,
            available: amount("70.0"),
            held: amount("30.0"),
            locked: false,
        });
        let tx_repo = InMemoryTransactionRepo::new();
        let dispute_repo = InMemoryDisputeRepo::new();

        let mut use_case = super::ChargebackUseCase::new(account_repo, tx_repo, dispute_repo);
        use_case.execute(1, 999);

        let account = use_case.account_repo().get(1).unwrap();
        assert_eq!(account.available, amount("70.0"));
        assert_eq!(account.held, amount("30.0"));
        assert!(!account.locked);
    }

    #[test]
    fn total_funds_decrease_by_disputed_amount() {
        let mut account_repo = InMemoryAccountRepo::new();
        account_repo.save(Account {
            client: 1,
            available: amount("50.0"),
            held: amount("80.0"),
            locked: false,
        });

        let mut tx_repo = InMemoryTransactionRepo::new();
        tx_repo.save(Transaction {
            tx_type: TransactionType::Deposit,
            client: 1,
            tx: 10,
            amount: Some(amount("25.0")),
        });

        let mut dispute_repo = InMemoryDisputeRepo::new();
        dispute_repo.mark_disputed(10);

        let mut use_case = super::ChargebackUseCase::new(account_repo, tx_repo, dispute_repo);
        use_case.execute(1, 10);

        let account = use_case.account_repo().get(1).unwrap();
        assert_eq!(account.available, amount("50.0"));
        assert_eq!(account.held, amount("55.0"));
        assert_eq!(account.total(), amount("105.0"));
        assert!(account.locked);
    }

    #[test]
    fn removes_dispute_after_chargeback() {
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

        let mut use_case = super::ChargebackUseCase::new(account_repo, tx_repo, dispute_repo);
        use_case.execute(1, 42);

        assert!(!use_case.dispute_repo().is_disputed(42));
    }
}
