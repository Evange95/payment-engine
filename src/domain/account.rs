use crate::domain::amount::Amount;

#[derive(Debug, Clone)]
pub struct Account {
    pub client: u16,
    pub available: Amount,
    pub held: Amount,
    pub locked: bool,
}

impl Account {
    pub fn new(client_id: u16) -> Self {
        Self {
            client: client_id,
            available: Amount::ZERO,
            held: Amount::ZERO,
            locked: false,
        }
    }

    pub fn total(&self) -> Amount {
        self.available + self.held
    }
}
