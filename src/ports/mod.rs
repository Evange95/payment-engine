use crate::application::use_cases::chargeback::ChargebackError;
use crate::application::use_cases::deposit::DepositError;
use crate::application::use_cases::dispute::DisputeError;
use crate::application::use_cases::resolve::ResolveError;
use crate::application::use_cases::withdrawal::WithdrawalError;
use crate::domain::account::Account;
use crate::domain::amount::Amount;
use crate::domain::transaction::Transaction;
use std::io;

#[cfg_attr(test, mockall::automock)]
pub trait AccountRepository {
    fn find_by_client_id(&self, client_id: u16) -> Option<Account>;
    fn save(&mut self, account: Account);
    fn all(&self) -> Vec<Account>;
}

#[cfg_attr(test, mockall::automock)]
pub trait TransactionRepository {
    fn find_by_tx_id(&self, tx_id: u32) -> Option<Transaction>;
    fn save(&mut self, transaction: Transaction);
}

#[cfg_attr(test, mockall::automock)]
pub trait DisputeRepository {
    fn is_disputed(&self, tx_id: u32) -> bool;
    fn mark_disputed(&mut self, tx_id: u32);
    fn remove_dispute(&mut self, tx_id: u32);
}

#[cfg_attr(test, mockall::automock)]
pub trait Deposit {
    fn execute(&mut self, client_id: u16, tx: u32, amount: Amount) -> Result<Account, DepositError>;
}

#[cfg_attr(test, mockall::automock)]
pub trait Withdraw {
    fn execute(
        &mut self,
        client_id: u16,
        tx: u32,
        amount: Amount,
    ) -> Result<(), WithdrawalError>;
}

#[cfg_attr(test, mockall::automock)]
pub trait DisputeTx {
    fn execute(&mut self, client_id: u16, tx_id: u32) -> Result<Option<Account>, DisputeError>;
}

#[cfg_attr(test, mockall::automock)]
pub trait Resolve {
    fn execute(&mut self, client_id: u16, tx_id: u32) -> Result<Option<Account>, ResolveError>;
}

#[cfg_attr(test, mockall::automock)]
pub trait Chargeback {
    fn execute(&mut self, client_id: u16, tx_id: u32) -> Result<Option<Account>, ChargebackError>;
}

pub trait AccountWriter {
    fn write_all(&mut self, accounts: &[Account]) -> Result<(), io::Error>;
}
