use crate::domain::account::Account;
use crate::ports::AccountWriter;
use serde::Serialize;
use std::io;

#[derive(Serialize)]
struct AccountRow {
    client: u16,
    available: String,
    held: String,
    total: String,
    locked: bool,
}

impl From<&Account> for AccountRow {
    fn from(a: &Account) -> Self {
        Self {
            client: a.client,
            available: a.available.to_string(),
            held: a.held.to_string(),
            total: a.total().to_string(),
            locked: a.locked,
        }
    }
}

pub struct CsvAccountWriter<W: io::Write> {
    writer: csv::Writer<W>,
}

impl<W: io::Write> CsvAccountWriter<W> {
    pub fn new(output: W) -> Self {
        Self {
            writer: csv::Writer::from_writer(output),
        }
    }
}

impl<W: io::Write> AccountWriter for CsvAccountWriter<W> {
    fn write_all(&mut self, accounts: &[Account]) -> Result<(), io::Error> {
        for account in accounts {
            self.writer
                .serialize(AccountRow::from(account))
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        }
        self.writer.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::account::Account;
    use crate::domain::amount::Amount;
    use crate::ports::AccountWriter;

    fn amount(s: &str) -> Amount {
        s.parse().unwrap()
    }

    #[test]
    fn writes_single_account_as_csv() {
        let accounts = vec![Account {
            client: 1,
            available: amount("1.5"),
            held: amount("0.0"),
            locked: false,
        }];

        let mut buf = Vec::new();
        {
            let mut writer = super::CsvAccountWriter::new(&mut buf);
            writer.write_all(&accounts).unwrap();
        }

        let output = String::from_utf8(buf).unwrap();
        assert_eq!(
            output,
            "client,available,held,total,locked\n1,1.50,0.00,1.50,false\n"
        );
    }
}
