mod adapters;
mod application;
mod domain;
mod ports;

use adapters::csv_reader::CsvTransactionReader;
use adapters::csv_writer::CsvAccountWriter;
use adapters::in_memory_account_repo::InMemoryAccountRepo;
use adapters::in_memory_dispute_repo::InMemoryDisputeRepo;
use adapters::in_memory_transaction_repo::InMemoryTransactionRepo;
use application::transaction_manager::TransactionManager;
use application::use_cases::chargeback::ChargebackUseCase;
use application::use_cases::deposit::DepositUseCase;
use application::use_cases::dispute::DisputeUseCase;
use application::use_cases::resolve::ResolveUseCase;
use application::use_cases::withdrawal::WithdrawalUseCase;
use ports::{AccountRepository, AccountWriter, TransactionReader};
use std::cell::RefCell;
use std::fs::File;
use std::rc::Rc;

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("usage: payment-engine <file.csv>");
    let file = File::open(&path).expect("cannot open file");

    let account_repo = Rc::new(RefCell::new(InMemoryAccountRepo::new()));
    let tx_repo = Rc::new(RefCell::new(InMemoryTransactionRepo::new()));
    let dispute_repo = Rc::new(RefCell::new(InMemoryDisputeRepo::new()));

    let deposit = DepositUseCase::new(account_repo.clone(), tx_repo.clone());
    let withdraw = WithdrawalUseCase::new(account_repo.clone(), tx_repo.clone());
    let dispute = DisputeUseCase::new(account_repo.clone(), tx_repo.clone(), dispute_repo.clone());
    let resolve = ResolveUseCase::new(account_repo.clone(), tx_repo.clone(), dispute_repo.clone());
    let chargeback =
        ChargebackUseCase::new(account_repo.clone(), tx_repo.clone(), dispute_repo.clone());

    let mut manager = TransactionManager::new(deposit, withdraw, dispute, resolve, chargeback);

    let reader = CsvTransactionReader::new(file);
    for tx in reader.read_all() {
        manager.process(tx);
    }

    let accounts = account_repo.borrow().all();
    let mut writer = CsvAccountWriter::new(std::io::stdout());
    writer.write_all(&accounts).unwrap();
}
