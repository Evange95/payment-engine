pub trait AccountRepository {
    fn find_by_client_id(&self, client_id: u16) -> Option<crate::domain::account::Account>;
    fn save(&mut self, account: crate::domain::account::Account);
}

pub trait TransactionRepository {
    fn find_by_tx_id(&self, tx_id: u32) -> Option<crate::domain::transaction::Transaction>;
    fn save(&mut self, transaction: crate::domain::transaction::Transaction);
}

pub trait DisputeRepository {
    fn is_disputed(&self, tx_id: u32) -> bool;
    fn mark_disputed(&mut self, tx_id: u32);
    fn remove_dispute(&mut self, tx_id: u32);
}
