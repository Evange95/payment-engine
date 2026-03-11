use crate::domain::account::Account;
use crate::ports::{AccountRepository, DisputeRepository, DisputeTx, TransactionRepository};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DisputeError {
    #[error("account is frozen")]
    FrozenAccount,
}

pub struct DisputeUseCase<A: AccountRepository, T: TransactionRepository, D: DisputeRepository> {
    account_repo: A,
    tx_repo: T,
    dispute_repo: D,
}

impl<A: AccountRepository, T: TransactionRepository, D: DisputeRepository> DisputeUseCase<A, T, D> {
    pub fn new(account_repo: A, tx_repo: T, dispute_repo: D) -> Self {
        Self {
            account_repo,
            tx_repo,
            dispute_repo,
        }
    }

    pub fn execute(&mut self, client_id: u16, tx_id: u32) -> Result<Option<Account>, DisputeError> {
        let tx = match self.tx_repo.find_by_tx_id(tx_id) {
            Some(tx) => tx,
            None => return Ok(None),
        };
        let mut account = match self.account_repo.find_by_client_id(client_id) {
            Some(a) => a,
            None => return Ok(None),
        };
        if account.locked {
            return Err(DisputeError::FrozenAccount);
        }

        let disputed_amount = match tx.amount {
            Some(a) => a,
            None => return Ok(None),
        };

        account.available = account.available - disputed_amount;
        account.held = account.held + disputed_amount;
        self.account_repo.save(account.clone());
        self.dispute_repo.mark_disputed(tx_id);
        Ok(Some(account))
    }
}

impl<A: AccountRepository, T: TransactionRepository, D: DisputeRepository> DisputeTx
    for DisputeUseCase<A, T, D>
{
    fn execute(&mut self, client_id: u16, tx_id: u32) -> Result<Option<Account>, DisputeError> {
        self.execute(client_id, tx_id)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::account::Account;
    use crate::domain::amount::Amount;
    use crate::domain::transaction::{Transaction, TransactionType};
    use crate::ports::{MockAccountRepository, MockDisputeRepository, MockTransactionRepository};

    fn amount(s: &str) -> Amount {
        s.parse().unwrap()
    }

    #[test]
    fn moves_funds_from_available_to_held() {
        let mut account_repo = MockAccountRepository::new();
        account_repo.expect_find_by_client_id().returning(|_| {
            Some(Account {
                client: 1,
                available: amount("100.0"),
                held: Amount::ZERO,
                locked: false,
            })
        });
        account_repo.expect_save().returning(|_| ());

        let mut tx_repo = MockTransactionRepository::new();
        tx_repo.expect_find_by_tx_id().returning(|_| {
            Some(Transaction {
                tx_type: TransactionType::Deposit,
                client: 1,
                tx: 42,
                amount: Some(amount("30.0")),
            })
        });

        let mut dispute_repo = MockDisputeRepository::new();
        dispute_repo.expect_mark_disputed().returning(|_| ());

        let mut use_case = super::DisputeUseCase::new(account_repo, tx_repo, dispute_repo);
        let account = use_case.execute(1, 42).unwrap().unwrap();

        assert_eq!(account.available, amount("70.0"));
        assert_eq!(account.held, amount("30.0"));
        assert_eq!(account.total(), amount("100.0"));
    }

    #[test]
    fn marks_transaction_as_disputed() {
        let mut account_repo = MockAccountRepository::new();
        account_repo.expect_find_by_client_id().returning(|_| {
            Some(Account {
                client: 1,
                available: amount("100.0"),
                held: Amount::ZERO,
                locked: false,
            })
        });
        account_repo.expect_save().returning(|_| ());

        let mut tx_repo = MockTransactionRepository::new();
        tx_repo.expect_find_by_tx_id().returning(|_| {
            Some(Transaction {
                tx_type: TransactionType::Deposit,
                client: 1,
                tx: 42,
                amount: Some(amount("30.0")),
            })
        });

        let mut dispute_repo = MockDisputeRepository::new();
        dispute_repo
            .expect_mark_disputed()
            .with(mockall::predicate::eq(42))
            .times(1)
            .returning(|_| ());

        let mut use_case = super::DisputeUseCase::new(account_repo, tx_repo, dispute_repo);
        use_case.execute(1, 42).unwrap();
    }

    #[test]
    fn ignores_non_existent_transaction() {
        let mut account_repo = MockAccountRepository::new();
        account_repo.expect_find_by_client_id().returning(|_| {
            Some(Account {
                client: 1,
                available: amount("100.0"),
                held: Amount::ZERO,
                locked: false,
            })
        });
        account_repo.expect_save().times(0);

        let mut tx_repo = MockTransactionRepository::new();
        tx_repo.expect_find_by_tx_id().returning(|_| None);

        let dispute_repo = MockDisputeRepository::new();

        let mut use_case = super::DisputeUseCase::new(account_repo, tx_repo, dispute_repo);
        let result = use_case.execute(1, 999).unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn ignores_non_existent_account() {
        let mut account_repo = MockAccountRepository::new();
        account_repo.expect_find_by_client_id().returning(|_| None);
        account_repo.expect_save().times(0);

        let mut tx_repo = MockTransactionRepository::new();
        tx_repo.expect_find_by_tx_id().returning(|_| {
            Some(Transaction {
                tx_type: TransactionType::Deposit,
                client: 1,
                tx: 42,
                amount: Some(amount("30.0")),
            })
        });

        let dispute_repo = MockDisputeRepository::new();

        let mut use_case = super::DisputeUseCase::new(account_repo, tx_repo, dispute_repo);
        let result = use_case.execute(1, 42).unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn rejects_frozen_account() {
        let mut account_repo = MockAccountRepository::new();
        account_repo.expect_find_by_client_id().returning(|_| {
            Some(Account {
                client: 1,
                available: amount("100.0"),
                held: Amount::ZERO,
                locked: true,
            })
        });
        account_repo.expect_save().times(0);

        let mut tx_repo = MockTransactionRepository::new();
        tx_repo.expect_find_by_tx_id().returning(|_| {
            Some(Transaction {
                tx_type: TransactionType::Deposit,
                client: 1,
                tx: 42,
                amount: Some(amount("30.0")),
            })
        });

        let dispute_repo = MockDisputeRepository::new();

        let mut use_case = super::DisputeUseCase::new(account_repo, tx_repo, dispute_repo);
        let result = use_case.execute(1, 42);

        assert!(result.is_err());
    }

    #[test]
    fn total_funds_unchanged_with_existing_held() {
        let mut account_repo = MockAccountRepository::new();
        account_repo.expect_find_by_client_id().returning(|_| {
            Some(Account {
                client: 1,
                available: amount("200.0"),
                held: amount("50.0"),
                locked: false,
            })
        });
        account_repo.expect_save().returning(|_| ());

        let mut tx_repo = MockTransactionRepository::new();
        tx_repo.expect_find_by_tx_id().returning(|_| {
            Some(Transaction {
                tx_type: TransactionType::Deposit,
                client: 1,
                tx: 7,
                amount: Some(amount("25.0")),
            })
        });

        let mut dispute_repo = MockDisputeRepository::new();
        dispute_repo.expect_mark_disputed().returning(|_| ());

        let mut use_case = super::DisputeUseCase::new(account_repo, tx_repo, dispute_repo);
        let account = use_case.execute(1, 7).unwrap().unwrap();

        assert_eq!(account.available, amount("175.0"));
        assert_eq!(account.held, amount("75.0"));
        assert_eq!(account.total(), amount("250.0"));
    }
}
