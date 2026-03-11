use crate::domain::account::Account;
use crate::domain::transaction::{Transaction, TransactionType};
use crate::ports;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProcessError {
    #[error("deposit failed: {0}")]
    Deposit(#[from] crate::application::use_cases::deposit::DepositError),
    #[error("withdrawal failed: {0}")]
    Withdrawal(#[from] crate::application::use_cases::withdrawal::WithdrawalError),
    #[error("dispute failed: {0}")]
    Dispute(#[from] crate::application::use_cases::dispute::DisputeError),
    #[error("resolve failed: {0}")]
    Resolve(#[from] crate::application::use_cases::resolve::ResolveError),
    #[error("chargeback failed: {0}")]
    Chargeback(#[from] crate::application::use_cases::chargeback::ChargebackError),
}

pub struct TransactionManager<D, W, Di, R, C>
where
    D: ports::Deposit,
    W: ports::Withdraw,
    Di: ports::DisputeTx,
    R: ports::Resolve,
    C: ports::Chargeback,
{
    deposit: D,
    withdraw: W,
    dispute: Di,
    resolve: R,
    chargeback: C,
}

impl<D, W, Di, R, C> TransactionManager<D, W, Di, R, C>
where
    D: ports::Deposit,
    W: ports::Withdraw,
    Di: ports::DisputeTx,
    R: ports::Resolve,
    C: ports::Chargeback,
{
    pub fn new(deposit: D, withdraw: W, dispute: Di, resolve: R, chargeback: C) -> Self {
        Self {
            deposit,
            withdraw,
            dispute,
            resolve,
            chargeback,
        }
    }

    pub fn process(&mut self, tx: Transaction) -> Result<Option<Account>, ProcessError> {
        match tx.tx_type {
            TransactionType::Deposit => {
                let amount = match tx.amount {
                    Some(a) => a,
                    None => return Ok(None),
                };
                let account = self.deposit.execute(tx.client, tx.tx, amount)?;
                Ok(Some(account))
            }
            TransactionType::Withdrawal => {
                let amount = match tx.amount {
                    Some(a) => a,
                    None => return Ok(None),
                };
                self.withdraw.execute(tx.client, tx.tx, amount)?;
                Ok(None)
            }
            TransactionType::Dispute => {
                Ok(self.dispute.execute(tx.client, tx.tx)?)
            }
            TransactionType::Resolve => {
                Ok(self.resolve.execute(tx.client, tx.tx)?)
            }
            TransactionType::Chargeback => {
                Ok(self.chargeback.execute(tx.client, tx.tx)?)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::account::Account;
    use crate::domain::amount::Amount;
    use crate::domain::transaction::{Transaction, TransactionType};
    use crate::ports::{MockChargeback, MockDeposit, MockDisputeTx, MockResolve, MockWithdraw};

    fn amount(s: &str) -> Amount {
        s.parse().unwrap()
    }

    #[test]
    fn routes_deposit_to_deposit_use_case() {
        let mut mock_deposit = MockDeposit::new();
        mock_deposit
            .expect_execute()
            .withf(|client_id, tx, amount| {
                *client_id == 1 && *tx == 42 && *amount == "100.0".parse::<Amount>().unwrap()
            })
            .times(1)
            .returning(|client_id, _, amount| {
                Ok(Account {
                    client: client_id,
                    available: amount,
                    held: Amount::ZERO,
                    locked: false,
                })
            });

        let mut manager = super::TransactionManager::new(
            mock_deposit,
            MockWithdraw::new(),
            MockDisputeTx::new(),
            MockResolve::new(),
            MockChargeback::new(),
        );

        let tx = Transaction {
            tx_type: TransactionType::Deposit,
            client: 1,
            tx: 42,
            amount: Some(amount("100.0")),
        };

        let account = manager.process(tx).unwrap().unwrap();
        assert_eq!(account.client, 1);
        assert_eq!(account.available, amount("100.0"));
    }

    #[test]
    fn routes_withdrawal_to_withdraw_use_case() {
        let mut mock_withdraw = MockWithdraw::new();
        mock_withdraw
            .expect_execute()
            .withf(|client_id, tx, amount| {
                *client_id == 1 && *tx == 42 && *amount == "10.0".parse::<Amount>().unwrap()
            })
            .times(1)
            .returning(|_, _, _| Ok(()));

        let mut manager = super::TransactionManager::new(
            MockDeposit::new(),
            mock_withdraw,
            MockDisputeTx::new(),
            MockResolve::new(),
            MockChargeback::new(),
        );

        let tx = Transaction {
            tx_type: TransactionType::Withdrawal,
            client: 1,
            tx: 42,
            amount: Some(amount("10.0")),
        };

        let result = manager.process(tx).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn routes_dispute_to_dispute_use_case() {
        let mut mock_dispute = MockDisputeTx::new();
        mock_dispute
            .expect_execute()
            .withf(|client_id, tx_id| *client_id == 1 && *tx_id == 42)
            .times(1)
            .returning(|client_id, _| {
                Ok(Some(Account {
                    client: client_id,
                    available: Amount::ZERO,
                    held: Amount::ZERO,
                    locked: false,
                }))
            });

        let mut manager = super::TransactionManager::new(
            MockDeposit::new(),
            MockWithdraw::new(),
            mock_dispute,
            MockResolve::new(),
            MockChargeback::new(),
        );

        let tx = Transaction {
            tx_type: TransactionType::Dispute,
            client: 1,
            tx: 42,
            amount: None,
        };

        let account = manager.process(tx).unwrap().unwrap();
        assert_eq!(account.client, 1);
    }

    #[test]
    fn routes_resolve_to_resolve_use_case() {
        let mut mock_resolve = MockResolve::new();
        mock_resolve
            .expect_execute()
            .withf(|client_id, tx_id| *client_id == 1 && *tx_id == 42)
            .times(1)
            .returning(|client_id, _| {
                Ok(Some(Account {
                    client: client_id,
                    available: Amount::ZERO,
                    held: Amount::ZERO,
                    locked: false,
                }))
            });

        let mut manager = super::TransactionManager::new(
            MockDeposit::new(),
            MockWithdraw::new(),
            MockDisputeTx::new(),
            mock_resolve,
            MockChargeback::new(),
        );

        let tx = Transaction {
            tx_type: TransactionType::Resolve,
            client: 1,
            tx: 42,
            amount: None,
        };

        let account = manager.process(tx).unwrap().unwrap();
        assert_eq!(account.client, 1);
    }

    #[test]
    fn routes_chargeback_to_chargeback_use_case() {
        let mut mock_chargeback = MockChargeback::new();
        mock_chargeback
            .expect_execute()
            .withf(|client_id, tx_id| *client_id == 1 && *tx_id == 42)
            .times(1)
            .returning(|client_id, _| {
                Ok(Some(Account {
                    client: client_id,
                    available: Amount::ZERO,
                    held: Amount::ZERO,
                    locked: true,
                }))
            });

        let mut manager = super::TransactionManager::new(
            MockDeposit::new(),
            MockWithdraw::new(),
            MockDisputeTx::new(),
            MockResolve::new(),
            mock_chargeback,
        );

        let tx = Transaction {
            tx_type: TransactionType::Chargeback,
            client: 1,
            tx: 42,
            amount: None,
        };

        let account = manager.process(tx).unwrap().unwrap();
        assert_eq!(account.client, 1);
        assert!(account.locked);
    }
}
