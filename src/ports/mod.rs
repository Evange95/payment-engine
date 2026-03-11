pub trait AccountRepository {
    fn find_by_client_id(&self, client_id: u16) -> Option<crate::domain::account::Account>;
    fn save(&mut self, account: crate::domain::account::Account);
}
