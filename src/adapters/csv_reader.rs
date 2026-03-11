use crate::domain::amount::Amount;
use crate::domain::transaction::{Transaction, TransactionType};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct CsvRow {
    r#type: String,
    client: u16,
    tx: u32,
    amount: Option<String>,
}

pub struct CsvTransactionReader<R: std::io::Read> {
    reader: csv::Reader<R>,
}

impl<R: std::io::Read> CsvTransactionReader<R> {
    pub fn new(input: R) -> Self {
        Self {
            reader: csv::ReaderBuilder::new()
                .trim(csv::Trim::All)
                .from_reader(input),
        }
    }
}

fn parse_row(row: CsvRow) -> Option<Transaction> {
    let (tx_type, amount) = match row.r#type.trim() {
        "deposit" => {
            let amount: Amount = row.amount?.trim().parse().ok()?;
            (TransactionType::Deposit, Some(amount))
        }
        "withdrawal" => {
            let amount: Amount = row.amount?.trim().parse().ok()?;
            (TransactionType::Withdrawal, Some(amount))
        }
        "dispute" => (TransactionType::Dispute, None),
        "resolve" => (TransactionType::Resolve, None),
        "chargeback" => (TransactionType::Chargeback, None),
        _ => return None,
    };

    Some(Transaction {
        tx_type,
        client: row.client,
        tx: row.tx,
        amount,
    })
}

impl<R: std::io::Read> Iterator for CsvTransactionReader<R> {
    type Item = Transaction;

    fn next(&mut self) -> Option<Transaction> {
        loop {
            let result = self.reader.deserialize::<CsvRow>().next()?;
            let row = match result {
                Ok(r) => r,
                Err(_) => continue,
            };
            if let Some(tx) = parse_row(row) {
                return Some(tx);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::amount::Amount;
    use crate::domain::transaction::TransactionType;

    fn amount(s: &str) -> Amount {
        s.parse().unwrap()
    }

    #[test]
    fn reads_dispute_without_amount() {
        let csv = "type,client,tx,amount\ndispute,1,42,\n";
        let reader = super::CsvTransactionReader::new(csv.as_bytes());

        let txs = reader.collect::<Vec<_>>();

        assert_eq!(txs.len(), 1);
        assert_eq!(txs[0].tx_type, TransactionType::Dispute);
        assert_eq!(txs[0].client, 1);
        assert_eq!(txs[0].tx, 42);
        assert_eq!(txs[0].amount, None);
    }

    #[test]
    fn reads_multiple_transaction_types() {
        let csv = "type,client,tx,amount\ndeposit,1,1,1.0\ndeposit,2,2,2.0\ndispute,1,1,\n";
        let reader = super::CsvTransactionReader::new(csv.as_bytes());

        let txs = reader.collect::<Vec<_>>();

        assert_eq!(txs.len(), 3);
        assert_eq!(txs[0].tx_type, TransactionType::Deposit);
        assert_eq!(txs[1].tx_type, TransactionType::Deposit);
        assert_eq!(txs[2].tx_type, TransactionType::Dispute);
    }

    #[test]
    fn handles_whitespace_in_csv() {
        let csv = "type, client, tx, amount\ndeposit, 1, 1, 1.0\n";
        let reader = super::CsvTransactionReader::new(csv.as_bytes());

        let txs = reader.collect::<Vec<_>>();

        assert_eq!(txs.len(), 1);
        assert_eq!(txs[0].tx_type, TransactionType::Deposit);
        assert_eq!(txs[0].amount, Some(amount("1.0")));
    }

    #[test]
    fn reads_deposit_from_csv() {
        let csv = "type,client,tx,amount\ndeposit,1,1,1.0\n";
        let reader = super::CsvTransactionReader::new(csv.as_bytes());

        let txs = reader.collect::<Vec<_>>();

        assert_eq!(txs.len(), 1);
        assert_eq!(txs[0].tx_type, TransactionType::Deposit);
        assert_eq!(txs[0].client, 1);
        assert_eq!(txs[0].tx, 1);
        assert_eq!(txs[0].amount, Some(amount("1.0")));
    }

    #[test]
    fn skips_malformed_csv_rows() {
        let csv = "type,client,tx,amount\ndeposit,1,1,1.0\nnot,a,valid,row\ndeposit,2,2,2.0\n";
        let reader = super::CsvTransactionReader::new(csv.as_bytes());

        let txs: Vec<_> = reader.collect();

        assert_eq!(txs.len(), 2);
        assert_eq!(txs[0].client, 1);
        assert_eq!(txs[1].client, 2);
    }

    #[test]
    fn iterates_transactions_one_at_a_time() {
        let csv = "type,client,tx,amount\ndeposit,1,1,1.0\ndeposit,2,2,2.0\n";
        let mut reader = super::CsvTransactionReader::new(csv.as_bytes());

        let first = reader.next().unwrap();
        assert_eq!(first.client, 1);
        assert_eq!(first.tx, 1);

        let second = reader.next().unwrap();
        assert_eq!(second.client, 2);
        assert_eq!(second.tx, 2);

        assert!(reader.next().is_none());
    }
}
