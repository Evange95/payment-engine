use crate::domain::amount::Amount;
use crate::domain::transaction::{Transaction, TransactionType};
use crate::ports::TransactionReader;
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

impl<R: std::io::Read> TransactionReader for CsvTransactionReader<R> {
    fn read_all(mut self) -> Vec<Transaction> {
        let mut transactions = Vec::new();
        for result in self.reader.deserialize::<CsvRow>() {
            if let Ok(row) = result {
                if let Some(tx) = parse_row(row) {
                    transactions.push(tx);
                }
            }
        }
        transactions
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::amount::Amount;
    use crate::domain::transaction::TransactionType;
    use crate::ports::TransactionReader;

    fn amount(s: &str) -> Amount {
        s.parse().unwrap()
    }

    #[test]
    fn reads_dispute_without_amount() {
        let csv = "type,client,tx,amount\ndispute,1,42,\n";
        let reader = super::CsvTransactionReader::new(csv.as_bytes());

        let txs = reader.read_all();

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

        let txs = reader.read_all();

        assert_eq!(txs.len(), 3);
        assert_eq!(txs[0].tx_type, TransactionType::Deposit);
        assert_eq!(txs[1].tx_type, TransactionType::Deposit);
        assert_eq!(txs[2].tx_type, TransactionType::Dispute);
    }

    #[test]
    fn handles_whitespace_in_csv() {
        let csv = "type, client, tx, amount\ndeposit, 1, 1, 1.0\n";
        let reader = super::CsvTransactionReader::new(csv.as_bytes());

        let txs = reader.read_all();

        assert_eq!(txs.len(), 1);
        assert_eq!(txs[0].tx_type, TransactionType::Deposit);
        assert_eq!(txs[0].amount, Some(amount("1.0")));
    }

    #[test]
    fn reads_deposit_from_csv() {
        let csv = "type,client,tx,amount\ndeposit,1,1,1.0\n";
        let reader = super::CsvTransactionReader::new(csv.as_bytes());

        let txs = reader.read_all();

        assert_eq!(txs.len(), 1);
        assert_eq!(txs[0].tx_type, TransactionType::Deposit);
        assert_eq!(txs[0].client, 1);
        assert_eq!(txs[0].tx, 1);
        assert_eq!(txs[0].amount, Some(amount("1.0")));
    }
}
