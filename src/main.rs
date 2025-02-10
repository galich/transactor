mod account;
mod money;
mod processor;
mod transactions;

use account::ClientId;
use processor::Processor;
use std::{error::Error, io};
use transactions::{chargeback, deposit, dispute, resolve, withdraw, Transaction, TransactionId};

fn main() {
    let transactions = read_transactions().unwrap();
    let mut processor = Processor::default();
    let _audit_records: Vec<_> = processor.process(&transactions).collect();

    // print out accounts
    println!("client, available, held, total, locked");
    processor.accounts.iter().for_each(|(id, account)| {
        let Some(total) = account.total() else { return };
        println!(
            "{id}, {}, {}, {}, {}",
            account.available, account.held, total, account.locked
        );
    });
}

fn read_transactions() -> Result<Vec<Transaction>, Box<dyn Error>> {
    let input_file_path = std::env::args().skip(1).next().ok_or(io::Error::new(
        io::ErrorKind::NotFound,
        "must provide an input file path",
    ))?;
    let mut rdr = csv::Reader::from_path(input_file_path)?;

    Ok(rdr
        .records()
        .filter_map(|result| {
            let Ok(record) = result else {
                return None;
            };
            let Some(transaction_type) = record.get(0) else {
                return None;
            };
            let Some(Some(client_id)) = record.get(1).map(|s| s.trim().parse::<ClientId>().ok())
            else {
                return None;
            };
            let Some(Some(tx_id)) = record
                .get(2)
                .map(|s| s.trim().parse::<TransactionId>().ok())
            else {
                return None;
            };
            let amount = record
                .get(3)
                .map(|s| s.trim().parse::<f64>().ok())
                .flatten();

            match transaction_type {
                "deposit" => amount.map(|amount| deposit(client_id, tx_id, amount)),
                "withdrawal" => amount.map(|amount| withdraw(client_id, tx_id, amount)),
                "dispute" => Some(dispute(client_id, tx_id)),
                "resolve" => Some(resolve(client_id, tx_id)),
                "chargeback" => Some(chargeback(client_id, tx_id)),
                _ => None,
            }
        })
        .collect())
}
