use crate::domain::account::Account;
use crate::domain::amount::Amount;
use crate::ports::AccountRepository;

pub struct DepositUseCase<R: AccountRepository> {
    account_repo: R,
}

impl<R: AccountRepository> DepositUseCase<R> {
    pub fn new(account_repo: R) -> Self {
        Self { account_repo }
    }

    pub fn execute(&mut self, client_id: u16, amount: Amount) {
        let mut account = self
            .account_repo
            .find_by_client_id(client_id)
            .unwrap_or_else(|| Account::new(client_id));

        account.available = account.available + amount;
        self.account_repo.save(account);
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
    fn creates_account_on_first_deposit() {
        let repo = InMemoryAccountRepo::new();
        let mut use_case = super::DepositUseCase::new(repo);

        use_case.execute(1, amount("10.0"));

        let account = use_case.repo().get(1).expect("account should exist");
        assert_eq!(account.available, amount("10.0"));
        assert_eq!(account.held, Amount::ZERO);
        assert!(!account.locked);
    }
}
