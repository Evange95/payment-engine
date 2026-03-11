use crate::domain::account::Account;
use crate::domain::amount::Amount;
use crate::domain::transaction::{Transaction, TransactionType};
use crate::ports::{AccountRepository, Deposit, TransactionRepository};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DepositError {
    #[error("account is frozen")]
    FrozenAccount,
}

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

    pub fn execute(
        &mut self,
        client_id: u16,
        tx: u32,
        amount: Amount,
    ) -> Result<Account, DepositError> {
        let mut account = self
            .account_repo
            .find_by_client_id(client_id)
            .unwrap_or_else(|| Account::new(client_id));

        if account.locked {
            return Err(DepositError::FrozenAccount);
        }

        account.available = account.available + amount;
        self.account_repo.save(account.clone());

        self.tx_repo.save(Transaction {
            tx_type: TransactionType::Deposit,
            client: client_id,
            tx,
            amount: Some(amount),
        });

        Ok(account)
    }
}

impl<A: AccountRepository, T: TransactionRepository> Deposit for DepositUseCase<A, T> {
    fn execute(
        &mut self,
        client_id: u16,
        tx: u32,
        amount: Amount,
    ) -> Result<Account, DepositError> {
        self.execute(client_id, tx, amount)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::amount::Amount;
    use crate::domain::transaction::TransactionType;
    use crate::ports::{MockAccountRepository, MockTransactionRepository};

    fn amount(s: &str) -> Amount {
        s.parse().unwrap()
    }

    #[test]
    fn increases_available_and_total() {
        let mut account_repo = MockAccountRepository::new();
        account_repo.expect_find_by_client_id().returning(|_| {
            Some(crate::domain::account::Account {
                client: 1,
                available: amount("30.0"),
                held: amount("10.0"),
                locked: false,
            })
        });
        account_repo.expect_save().returning(|_| ());

        let mut tx_repo = MockTransactionRepository::new();
        tx_repo.expect_save().returning(|_| ());

        let mut use_case = super::DepositUseCase::new(account_repo, tx_repo);
        let account = use_case.execute(1, 1, amount("20.0")).unwrap();

        assert_eq!(account.available, amount("50.0"));
        assert_eq!(account.total(), amount("60.0"));
    }

    #[test]
    fn creates_account_on_first_deposit() {
        let mut account_repo = MockAccountRepository::new();
        account_repo.expect_find_by_client_id().returning(|_| None);
        account_repo.expect_save().returning(|_| ());

        let mut tx_repo = MockTransactionRepository::new();
        tx_repo.expect_save().returning(|_| ());

        let mut use_case = super::DepositUseCase::new(account_repo, tx_repo);
        let account = use_case.execute(1, 1, amount("10.0")).unwrap();

        assert_eq!(account.available, amount("10.0"));
        assert_eq!(account.held, Amount::ZERO);
        assert!(!account.locked);
    }

    #[test]
    fn adds_to_existing_available_balance() {
        let mut account_repo = MockAccountRepository::new();
        account_repo.expect_find_by_client_id().returning(|_| {
            Some(crate::domain::account::Account {
                client: 1,
                available: amount("50.0"),
                held: Amount::ZERO,
                locked: false,
            })
        });
        account_repo.expect_save().returning(|_| ());

        let mut tx_repo = MockTransactionRepository::new();
        tx_repo.expect_save().returning(|_| ());

        let mut use_case = super::DepositUseCase::new(account_repo, tx_repo);
        let account = use_case.execute(1, 1, amount("25.5")).unwrap();

        assert_eq!(account.available, amount("75.5"));
    }

    #[test]
    fn deposits_to_separate_clients_independently() {
        let mut account_repo = MockAccountRepository::new();
        account_repo.expect_find_by_client_id().returning(|_| None);
        account_repo.expect_save().returning(|_| ());

        let mut tx_repo = MockTransactionRepository::new();
        tx_repo.expect_save().returning(|_| ());

        let mut use_case = super::DepositUseCase::new(account_repo, tx_repo);
        let a1 = use_case.execute(1, 1, amount("100.0")).unwrap();
        let a2 = use_case.execute(2, 2, amount("200.0")).unwrap();

        assert_eq!(a1.available, amount("100.0"));
        assert_eq!(a2.available, amount("200.0"));
    }

    #[test]
    fn rejects_frozen_account() {
        let mut account_repo = MockAccountRepository::new();
        account_repo.expect_find_by_client_id().returning(|_| {
            Some(crate::domain::account::Account {
                client: 1,
                available: amount("100.0"),
                held: Amount::ZERO,
                locked: true,
            })
        });
        account_repo.expect_save().times(0);

        let tx_repo = MockTransactionRepository::new();

        let mut use_case = super::DepositUseCase::new(account_repo, tx_repo);
        let result = use_case.execute(1, 1, amount("10.0"));

        assert!(result.is_err());
    }

    #[test]
    fn saves_transaction_record() {
        let mut account_repo = MockAccountRepository::new();
        account_repo.expect_find_by_client_id().returning(|_| None);
        account_repo.expect_save().returning(|_| ());

        let mut tx_repo = MockTransactionRepository::new();
        tx_repo
            .expect_save()
            .withf(|tx| {
                tx.tx_type == TransactionType::Deposit
                    && tx.client == 1
                    && tx.tx == 42
                    && tx.amount == Some("10.0".parse().unwrap())
            })
            .times(1)
            .returning(|_| ());

        let mut use_case = super::DepositUseCase::new(account_repo, tx_repo);
        use_case.execute(1, 42, amount("10.0")).unwrap();
    }
}
