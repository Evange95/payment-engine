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
}
