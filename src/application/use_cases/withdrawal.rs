use crate::domain::amount::Amount;
use crate::ports::AccountRepository;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WithdrawalError {
    #[error("insufficient available funds")]
    InsufficientFunds,
}

pub struct WithdrawalUseCase<R: AccountRepository> {
    account_repo: R,
}

impl<R: AccountRepository> WithdrawalUseCase<R> {
    pub fn new(account_repo: R) -> Self {
        Self { account_repo }
    }

    pub fn execute(&mut self, client_id: u16, amount: Amount) -> Result<(), WithdrawalError> {
        let mut account = self
            .account_repo
            .find_by_client_id(client_id)
            .unwrap_or_else(|| crate::domain::account::Account::new(client_id));

        if (account.available - amount).is_negative() {
            return Err(WithdrawalError::InsufficientFunds);
        }

        account.available = account.available - amount;
        self.account_repo.save(account);
        Ok(())
    }

    pub fn repo(&self) -> &R {
        &self.account_repo
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::account::Account;
    use crate::domain::amount::Amount;
    use crate::ports::AccountRepository;
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
    fn decreases_available_and_total() {
        let mut repo = InMemoryAccountRepo::new();
        repo.save(Account {
            client: 2,
            available: amount("100.0"),
            held: amount("10.0"),
            locked: false,
        });
        let mut use_case = super::WithdrawalUseCase::new(repo);

        let result = use_case.execute(2, amount("1.0"));

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
        let mut use_case = super::WithdrawalUseCase::new(repo);

        let result = use_case.execute(2, amount("10.0"));

        assert!(result.is_err());
        let account = use_case.repo().get(2).unwrap();
        assert_eq!(account.available, amount("5.0"));
        assert_eq!(account.total(), amount("15.0"));
    }
}
