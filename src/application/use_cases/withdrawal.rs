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
}

impl<A: AccountRepository, T: TransactionRepository> Withdraw for WithdrawalUseCase<A, T> {
    fn execute(&mut self, client_id: u16, tx: u32, amount: Amount) -> Result<(), WithdrawalError> {
        self.execute(client_id, tx, amount)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::account::Account;
    use crate::domain::amount::Amount;
    use crate::domain::transaction::TransactionType;
    use crate::ports::{MockAccountRepository, MockTransactionRepository};

    fn amount(s: &str) -> Amount {
        s.parse().unwrap()
    }

    #[test]
    fn decreases_available_and_total() {
        let mut account_repo = MockAccountRepository::new();
        account_repo.expect_find_by_client_id().returning(|_| {
            Some(Account {
                client: 2,
                available: amount("100.0"),
                held: amount("10.0"),
                locked: false,
            })
        });
        account_repo
            .expect_save()
            .withf(|a| {
                a.available == "99.0".parse().unwrap() && a.total() == "109.0".parse().unwrap()
            })
            .times(1)
            .returning(|_| ());

        let mut tx_repo = MockTransactionRepository::new();
        tx_repo.expect_save().returning(|_| ());

        let mut use_case = super::WithdrawalUseCase::new(account_repo, tx_repo);
        let result = use_case.execute(2, 1, amount("1.0"));

        assert!(result.is_ok());
    }

    #[test]
    fn fails_with_insufficient_available_funds() {
        let mut account_repo = MockAccountRepository::new();
        account_repo.expect_find_by_client_id().returning(|_| {
            Some(Account {
                client: 2,
                available: amount("5.0"),
                held: amount("10.0"),
                locked: false,
            })
        });
        account_repo.expect_save().times(0);

        let tx_repo = MockTransactionRepository::new();

        let mut use_case = super::WithdrawalUseCase::new(account_repo, tx_repo);
        let result = use_case.execute(2, 1, amount("10.0"));

        assert!(result.is_err());
    }

    #[test]
    fn fails_on_non_existent_account() {
        let mut account_repo = MockAccountRepository::new();
        account_repo.expect_find_by_client_id().returning(|_| None);
        account_repo.expect_save().times(0);

        let tx_repo = MockTransactionRepository::new();

        let mut use_case = super::WithdrawalUseCase::new(account_repo, tx_repo);
        let result = use_case.execute(99, 1, amount("1.0"));

        assert!(result.is_err());
    }

    #[test]
    fn withdrawals_to_separate_clients_independently() {
        let mut account_repo = MockAccountRepository::new();
        account_repo.expect_find_by_client_id().returning(|id| {
            Some(Account {
                client: id,
                available: if id == 1 {
                    amount("100.0")
                } else {
                    amount("200.0")
                },
                held: Amount::ZERO,
                locked: false,
            })
        });
        account_repo
            .expect_save()
            .withf(|a| {
                (a.client == 1 && a.available == "90.0".parse().unwrap())
                    || (a.client == 2 && a.available == "150.0".parse().unwrap())
            })
            .times(2)
            .returning(|_| ());

        let mut tx_repo = MockTransactionRepository::new();
        tx_repo.expect_save().returning(|_| ());

        let mut use_case = super::WithdrawalUseCase::new(account_repo, tx_repo);
        use_case.execute(1, 1, amount("10.0")).unwrap();
        use_case.execute(2, 2, amount("50.0")).unwrap();
    }

    #[test]
    fn saves_transaction_record() {
        let mut account_repo = MockAccountRepository::new();
        account_repo.expect_find_by_client_id().returning(|_| {
            Some(Account {
                client: 2,
                available: amount("100.0"),
                held: Amount::ZERO,
                locked: false,
            })
        });
        account_repo.expect_save().returning(|_| ());

        let mut tx_repo = MockTransactionRepository::new();
        tx_repo
            .expect_save()
            .withf(|tx| {
                tx.tx_type == TransactionType::Withdrawal
                    && tx.client == 2
                    && tx.tx == 42
                    && tx.amount == Some("10.0".parse().unwrap())
            })
            .times(1)
            .returning(|_| ());

        let mut use_case = super::WithdrawalUseCase::new(account_repo, tx_repo);
        use_case.execute(2, 42, amount("10.0")).unwrap();
    }
}
