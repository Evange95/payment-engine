use crate::domain::account::Account;
use crate::ports::{
    AccountRepository, Chargeback, DisputeRepository, TransactionRepository,
};

pub struct ChargebackUseCase<A: AccountRepository, T: TransactionRepository, D: DisputeRepository> {
    account_repo: A,
    tx_repo: T,
    dispute_repo: D,
}

impl<A: AccountRepository, T: TransactionRepository, D: DisputeRepository>
    ChargebackUseCase<A, T, D>
{
    pub fn new(account_repo: A, tx_repo: T, dispute_repo: D) -> Self {
        Self {
            account_repo,
            tx_repo,
            dispute_repo,
        }
    }

    pub fn execute(&mut self, client_id: u16, tx_id: u32) -> Option<Account> {
        if !self.dispute_repo.is_disputed(tx_id) {
            return None;
        }

        let tx = self.tx_repo.find_by_tx_id(tx_id)?;
        let mut account = self.account_repo.find_by_client_id(client_id)?;
        let amount = tx.amount?;

        account.held = account.held - amount;
        account.locked = true;
        self.account_repo.save(account.clone());
        self.dispute_repo.remove_dispute(tx_id);
        Some(account)
    }
}

impl<A: AccountRepository, T: TransactionRepository, D: DisputeRepository> Chargeback
    for ChargebackUseCase<A, T, D>
{
    fn execute(&mut self, client_id: u16, tx_id: u32) -> Option<Account> {
        self.execute(client_id, tx_id)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::account::Account;
    use crate::domain::amount::Amount;
    use crate::domain::transaction::{Transaction, TransactionType};
    use crate::ports::{MockAccountRepository, MockDisputeRepository, MockTransactionRepository};

    fn amount(s: &str) -> Amount {
        s.parse().unwrap()
    }

    #[test]
    fn withdraws_held_funds_and_freezes_account() {
        let mut account_repo = MockAccountRepository::new();
        account_repo.expect_find_by_client_id().returning(|_| {
            Some(Account {
                client: 1,
                available: amount("70.0"),
                held: amount("30.0"),
                locked: false,
            })
        });
        account_repo.expect_save().returning(|_| ());

        let mut tx_repo = MockTransactionRepository::new();
        tx_repo.expect_find_by_tx_id().returning(|_| {
            Some(Transaction {
                tx_type: TransactionType::Deposit,
                client: 1,
                tx: 42,
                amount: Some(amount("30.0")),
            })
        });

        let mut dispute_repo = MockDisputeRepository::new();
        dispute_repo.expect_is_disputed().returning(|_| true);
        dispute_repo.expect_remove_dispute().returning(|_| ());

        let mut use_case = super::ChargebackUseCase::new(account_repo, tx_repo, dispute_repo);
        let account = use_case.execute(1, 42).unwrap();

        assert_eq!(account.available, amount("70.0"));
        assert_eq!(account.held, Amount::ZERO);
        assert_eq!(account.total(), amount("70.0"));
        assert!(account.locked);
    }

    #[test]
    fn ignores_non_disputed_transaction() {
        let mut account_repo = MockAccountRepository::new();
        account_repo.expect_save().times(0);

        let tx_repo = MockTransactionRepository::new();

        let mut dispute_repo = MockDisputeRepository::new();
        dispute_repo.expect_is_disputed().returning(|_| false);

        let mut use_case = super::ChargebackUseCase::new(account_repo, tx_repo, dispute_repo);
        let result = use_case.execute(1, 42);

        assert!(result.is_none());
    }

    #[test]
    fn ignores_non_existent_transaction() {
        let mut account_repo = MockAccountRepository::new();
        account_repo.expect_save().times(0);

        let mut tx_repo = MockTransactionRepository::new();
        tx_repo.expect_find_by_tx_id().returning(|_| None);

        let mut dispute_repo = MockDisputeRepository::new();
        dispute_repo.expect_is_disputed().returning(|_| true);

        let mut use_case = super::ChargebackUseCase::new(account_repo, tx_repo, dispute_repo);
        let result = use_case.execute(1, 999);

        assert!(result.is_none());
    }

    #[test]
    fn total_funds_decrease_by_disputed_amount() {
        let mut account_repo = MockAccountRepository::new();
        account_repo.expect_find_by_client_id().returning(|_| {
            Some(Account {
                client: 1,
                available: amount("50.0"),
                held: amount("80.0"),
                locked: false,
            })
        });
        account_repo.expect_save().returning(|_| ());

        let mut tx_repo = MockTransactionRepository::new();
        tx_repo.expect_find_by_tx_id().returning(|_| {
            Some(Transaction {
                tx_type: TransactionType::Deposit,
                client: 1,
                tx: 10,
                amount: Some(amount("25.0")),
            })
        });

        let mut dispute_repo = MockDisputeRepository::new();
        dispute_repo.expect_is_disputed().returning(|_| true);
        dispute_repo.expect_remove_dispute().returning(|_| ());

        let mut use_case = super::ChargebackUseCase::new(account_repo, tx_repo, dispute_repo);
        let account = use_case.execute(1, 10).unwrap();

        assert_eq!(account.available, amount("50.0"));
        assert_eq!(account.held, amount("55.0"));
        assert_eq!(account.total(), amount("105.0"));
        assert!(account.locked);
    }

    #[test]
    fn removes_dispute_after_chargeback() {
        let mut account_repo = MockAccountRepository::new();
        account_repo.expect_find_by_client_id().returning(|_| {
            Some(Account {
                client: 1,
                available: amount("70.0"),
                held: amount("30.0"),
                locked: false,
            })
        });
        account_repo.expect_save().returning(|_| ());

        let mut tx_repo = MockTransactionRepository::new();
        tx_repo.expect_find_by_tx_id().returning(|_| {
            Some(Transaction {
                tx_type: TransactionType::Deposit,
                client: 1,
                tx: 42,
                amount: Some(amount("30.0")),
            })
        });

        let mut dispute_repo = MockDisputeRepository::new();
        dispute_repo.expect_is_disputed().returning(|_| true);
        dispute_repo
            .expect_remove_dispute()
            .with(mockall::predicate::eq(42))
            .times(1)
            .returning(|_| ());

        let mut use_case = super::ChargebackUseCase::new(account_repo, tx_repo, dispute_repo);
        use_case.execute(1, 42);
    }
}
