use crate::domain::amount::Amount;
use crate::domain::transaction::{Transaction, TransactionType};
use crate::ports::{AccountRepository, TransactionRepository, Withdraw};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WithdrawalError {
    #[error("insufficient available funds")]
    InsufficientFunds,
}

pub struct WithdrawalUseCase<A: AccountRepository, T: TransactionRepository> {
    account_repo: A,
    tx_repo: T,
}

impl<A: AccountRepository, T: TransactionRepository> WithdrawalUseCase<A, T> {
    pub fn new(account_repo: A, tx_repo: T) -> Self {
        Self {
            account_repo,
            tx_repo,
        }
    }

    pub fn execute(
        &mut self,
        client_id: u16,
        tx: u32,
        amount: Amount,
    ) -> Result<(), WithdrawalError> {
        let mut account = self
            .account_repo
            .find_by_client_id(client_id)
            .unwrap_or_else(|| crate::domain::account::Account::new(client_id));

        if (account.available - amount).is_negative() {
            return Err(WithdrawalError::InsufficientFunds);
        }

        account.available = account.available - amount;
        self.account_repo.save(account);

        self.tx_repo.save(Transaction {
            tx_type: TransactionType::Withdrawal,
            client: client_id,
            tx,
            amount: Some(amount),
        });

        Ok(())
    }

    pub fn repo(&self) -> &A {
        &self.account_repo
    }

    pub fn tx_repo(&self) -> &T {
        &self.tx_repo
    }
}

impl<A: AccountRepository, T: TransactionRepository> Withdraw for WithdrawalUseCase<A, T> {
    fn execute(
        &mut self,
        client_id: u16,
        tx: u32,
        amount: Amount,
    ) -> Result<(), WithdrawalError> {
        self.execute(client_id, tx, amount)
    }
}

#[cfg(test)]
mod tests {
    use crate::adapters::in_memory_account_repo::InMemoryAccountRepo;
    use crate::adapters::in_memory_transaction_repo::InMemoryTransactionRepo;
    use crate::domain::account::Account;
    use crate::domain::amount::Amount;
    use crate::domain::transaction::TransactionType;
    use crate::ports::AccountRepository;

    fn amount(s: &str) -> Amount {
        s.parse().unwrap()
    }

    #[test]
    fn decreases_available_and_total() {
        let mut repo = InMemoryAccountRepo::new();
        repo.save(Account {
            client: 2,
            available: amount("100.0"),
            held: amount("10.0"),
            locked: false,
        });
        let mut use_case = super::WithdrawalUseCase::new(repo, InMemoryTransactionRepo::new());

        let result = use_case.execute(2, 1, amount("1.0"));

        assert!(result.is_ok());
        let account = use_case.repo().get(2).unwrap();
        assert_eq!(account.available, amount("99.0"));
        assert_eq!(account.total(), amount("109.0"));
    }

    #[test]
    fn fails_with_insufficient_available_funds() {
        let mut repo = InMemoryAccountRepo::new();
        repo.save(Account {
            client: 2,
            available: amount("5.0"),
            held: amount("10.0"),
            locked: false,
        });
        let mut use_case = super::WithdrawalUseCase::new(repo, InMemoryTransactionRepo::new());

        let result = use_case.execute(2, 1, amount("10.0"));

        assert!(result.is_err());
        let account = use_case.repo().get(2).unwrap();
        assert_eq!(account.available, amount("5.0"));
        assert_eq!(account.total(), amount("15.0"));
    }

    #[test]
    fn fails_on_non_existent_account() {
        let repo = InMemoryAccountRepo::new();
        let mut use_case = super::WithdrawalUseCase::new(repo, InMemoryTransactionRepo::new());

        let result = use_case.execute(99, 1, amount("1.0"));

        assert!(result.is_err());
    }

    #[test]
    fn withdrawals_to_separate_clients_independently() {
        let mut repo = InMemoryAccountRepo::new();
        repo.save(Account {
            client: 1,
            available: amount("100.0"),
            held: Amount::ZERO,
            locked: false,
        });
        repo.save(Account {
            client: 2,
            available: amount("200.0"),
            held: Amount::ZERO,
            locked: false,
        });
        let mut use_case = super::WithdrawalUseCase::new(repo, InMemoryTransactionRepo::new());

        use_case.execute(1, 1, amount("10.0")).unwrap();
        use_case.execute(2, 2, amount("50.0")).unwrap();

        assert_eq!(use_case.repo().get(1).unwrap().available, amount("90.0"));
        assert_eq!(use_case.repo().get(2).unwrap().available, amount("150.0"));
    }

    #[test]
    fn saves_transaction_record() {
        let mut repo = InMemoryAccountRepo::new();
        repo.save(Account {
            client: 2,
            available: amount("100.0"),
            held: Amount::ZERO,
            locked: false,
        });
        let mut use_case = super::WithdrawalUseCase::new(repo, InMemoryTransactionRepo::new());

        use_case.execute(2, 42, amount("10.0")).unwrap();

        let tx = use_case
            .tx_repo()
            .get(42)
            .expect("transaction should be saved");
        assert_eq!(tx.tx_type, TransactionType::Withdrawal);
        assert_eq!(tx.client, 2);
        assert_eq!(tx.tx, 42);
        assert_eq!(tx.amount, Some(amount("10.0")));
    }
}
