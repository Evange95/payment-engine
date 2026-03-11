use crate::domain::account::Account;
use crate::ports::AccountRepository;
use std::collections::HashMap;

pub struct InMemoryAccountRepo {
    accounts: HashMap<u16, Account>,
}

impl InMemoryAccountRepo {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
        }
    }

    pub fn get(&self, client_id: u16) -> Option<&Account> {
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

    fn all(&self) -> Vec<Account> {
        self.accounts.values().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::account::Account;
    use crate::domain::amount::Amount;
    use crate::ports::AccountRepository;

    fn amount(s: &str) -> Amount {
        s.parse().unwrap()
    }

    #[test]
    fn all_returns_all_saved_accounts() {
        let mut repo = super::InMemoryAccountRepo::new();
        repo.save(Account {
            client: 1,
            available: amount("10.0"),
            held: Amount::ZERO,
            locked: false,
        });
        repo.save(Account {
            client: 2,
            available: amount("20.0"),
            held: Amount::ZERO,
            locked: false,
        });

        let mut accounts = repo.all();
        accounts.sort_by_key(|a| a.client);

        assert_eq!(accounts.len(), 2);
        assert_eq!(accounts[0].client, 1);
        assert_eq!(accounts[1].client, 2);
    }
}
