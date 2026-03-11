use crate::domain::amount::Amount;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Clone)]
pub struct Transaction {
    pub tx_type: TransactionType,
    pub client: u16,
    pub tx: u32,
    pub amount: Option<Amount>,
}
