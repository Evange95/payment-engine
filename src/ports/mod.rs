use crate::domain::account::Account;
use crate::domain::amount::Amount;
use crate::domain::transaction::Transaction;
use std::io;

pub trait AccountRepository {
    fn find_by_client_id(&self, client_id: u16) -> Option<Account>;
    fn save(&mut self, account: Account);
    fn all(&self) -> Vec<Account>;
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

#[cfg_attr(test, mockall::automock)]
pub trait Deposit {
    fn execute(&mut self, client_id: u16, tx: u32, amount: Amount) -> Account;
}

#[cfg_attr(test, mockall::automock)]
pub trait Withdraw {
    fn execute(
        &mut self,
        client_id: u16,
        tx: u32,
        amount: Amount,
    ) -> Result<(), crate::application::use_cases::withdrawal::WithdrawalError>;
}

#[cfg_attr(test, mockall::automock)]
pub trait DisputeTx {
    fn execute(&mut self, client_id: u16, tx_id: u32) -> Option<Account>;
}

#[cfg_attr(test, mockall::automock)]
pub trait Resolve {
    fn execute(&mut self, client_id: u16, tx_id: u32) -> Option<Account>;
}

#[cfg_attr(test, mockall::automock)]
pub trait Chargeback {
    fn execute(&mut self, client_id: u16, tx_id: u32) -> Option<Account>;
}

pub trait TransactionReader {
    fn read_all(self) -> Vec<Transaction>;
}

pub trait AccountWriter {
    fn write_all(&mut self, accounts: &[Account]) -> Result<(), io::Error>;
}
