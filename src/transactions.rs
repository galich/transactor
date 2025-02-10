use crate::{account::ClientId, money::MoneyAmount};

pub type TransactionId = u32;

#[derive(Debug)]
pub enum TransactionDetail {
    Deposit { amount: MoneyAmount },
    Withdrawal { amount: MoneyAmount },
    Dispute { tx_id: TransactionId },
    Resolve { tx_id: TransactionId },
    ChargeBack { tx_id: TransactionId },
}

#[derive(Debug)]
pub struct Transaction {
    pub id: TransactionId,
    pub client_id: ClientId,
    pub detail: TransactionDetail,
}

pub fn deposit(
    client_id: ClientId,
    tx_id: TransactionId,
    amount: impl Into<MoneyAmount>,
) -> Transaction {
    Transaction {
        client_id,
        id: tx_id,
        detail: TransactionDetail::Deposit {
            amount: amount.into(),
        },
    }
}

pub fn withdraw(
    client_id: ClientId,
    tx_id: TransactionId,
    amount: impl Into<MoneyAmount>,
) -> Transaction {
    Transaction {
        client_id,
        id: tx_id,
        detail: TransactionDetail::Withdrawal {
            amount: amount.into(),
        },
    }
}

pub fn dispute(client_id: ClientId, disputed_tx_id: TransactionId) -> Transaction {
    Transaction {
        client_id,
        id: 0, // For simplicity do not track tx_id of dispute
        detail: TransactionDetail::Dispute {
            tx_id: disputed_tx_id,
        },
    }
}

pub fn resolve(client_id: ClientId, disputed_tx_id: TransactionId) -> Transaction {
    Transaction {
        client_id,
        id: 0, // For simplicity do not track tx_id of resolve
        detail: TransactionDetail::Resolve {
            tx_id: disputed_tx_id,
        },
    }
}

pub fn chargeback(client_id: ClientId, disputed_tx_id: TransactionId) -> Transaction {
    Transaction {
        client_id,
        id: 0, // For simplicity do not track tx_id of chargeback
        detail: TransactionDetail::ChargeBack {
            tx_id: disputed_tx_id,
        },
    }
}
