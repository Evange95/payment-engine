use crate::domain::account::Account;
use crate::domain::transaction::{Transaction, TransactionType};
use crate::ports::{AccountRepository, DisputeRepository, TransactionRepository};

pub struct TransactionManager<A: AccountRepository, T: TransactionRepository, D: DisputeRepository>
{
    account_repo: A,
    tx_repo: T,
    dispute_repo: D,
}

impl<A: AccountRepository, T: TransactionRepository, D: DisputeRepository>
    TransactionManager<A, T, D>
{
    pub fn new(account_repo: A, tx_repo: T, dispute_repo: D) -> Self {
        Self {
            account_repo,
            tx_repo,
            dispute_repo,
        }
    }

    pub fn process(&mut self, tx: Transaction) -> Option<Account> {
        match tx.tx_type {
            TransactionType::Deposit => {
                let amount = tx.amount?;
                let mut account = self
                    .account_repo
                    .find_by_client_id(tx.client)
                    .unwrap_or_else(|| Account::new(tx.client));

                account.available = account.available + amount;
                self.account_repo.save(account.clone());

                self.tx_repo.save(Transaction {
                    tx_type: TransactionType::Deposit,
                    client: tx.client,
                    tx: tx.tx,
                    amount: Some(amount),
                });

                Some(account)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::adapters::in_memory_account_repo::InMemoryAccountRepo;
    use crate::adapters::in_memory_dispute_repo::InMemoryDisputeRepo;
    use crate::adapters::in_memory_transaction_repo::InMemoryTransactionRepo;
    use crate::domain::amount::Amount;
    use crate::domain::transaction::{Transaction, TransactionType};

    fn amount(s: &str) -> Amount {
        s.parse().unwrap()
    }

    #[test]
    fn processes_deposit_and_returns_account() {
        let mut manager = super::TransactionManager::new(
            InMemoryAccountRepo::new(),
            InMemoryTransactionRepo::new(),
            InMemoryDisputeRepo::new(),
        );

        let tx = Transaction {
            tx_type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Some(amount("100.0")),
        };

        let account = manager.process(tx).unwrap();
        assert_eq!(account.client, 1);
        assert_eq!(account.available, amount("100.0"));
        assert_eq!(account.held, Amount::ZERO);
        assert_eq!(account.total(), amount("100.0"));
    }
}
