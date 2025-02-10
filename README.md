# A toy payment processor

To build `cargo build`

To test `cargo test`

To run `cargo run -- input.csv`

## Notes and assumptions

* Money amount
    * is using fixed point integer (i64 or i128)
    * limit is around 900 trillion
    * while external fixed integer crate could be have been used, this project uses own simplified implementation of it
* CSV
    * for simplicity silently ignores invalid input
    * dispute, resolve and chargeback must have at least , or any value in place of amount
* Transactions
    * for simplicity do not track tx_id of dispute, resolve or chargeback
    * withdrawals are prohibited from locked accounts, but deposit and dispute related transactions are allowed
* Account
    * Tracks own disputes to avoid mixing with other accounts
    * Generate audit records that are used in tests, but can also be used at runtime
