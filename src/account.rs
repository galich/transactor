use crate::{money::MoneyAmount, transactions::TransactionId};
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum AuditRecord {
    Processed,
    CanNotDepositNegative,
    CanNotWithdrawNegative,
    NotEnoughMoneyToWithdraw,
    DisputedDepositNotFound,
    NotEnoughMoneyToRelease,
    NotEnoughMoneyToChargeBack,
    MoneyOverflow,
    MoneyUnderflow,
    DisputeNotFound,
    AccountLocked,
}

pub type ClientId = u16;

#[derive(Debug, Default)]
pub struct Account {
    pub available: MoneyAmount,
    pub held: MoneyAmount,
    pub locked: bool,

    /// Amounts of previously seen transactions in this account (for disputes)
    pub deposited_amounts: HashMap<TransactionId, MoneyAmount>,

    /// Amounts that are under active dispute
    pub disputed_amounts: HashMap<TransactionId, MoneyAmount>,
}

impl Account {
    pub fn total(&self) -> Option<MoneyAmount> {
        self.available.try_change(self.held)
    }

    /// Deposit money to the account
    pub fn deposit(&mut self, tx_id: TransactionId, amount: MoneyAmount) -> AuditRecord {
        if amount < 0 {
            return AuditRecord::CanNotDepositNegative;
        }

        let Some(new_available) = self.available.try_change(amount) else {
            return AuditRecord::MoneyOverflow;
        };

        self.available = new_available;
        self.deposited_amounts.insert(tx_id, amount);

        AuditRecord::Processed
    }

    /// Withdraw money from the account
    pub fn withdraw(&mut self, amount: MoneyAmount) -> AuditRecord {
        if amount < 0 {
            return AuditRecord::CanNotWithdrawNegative;
        }

        if self.locked {
            return AuditRecord::AccountLocked;
        }

        if self.available < amount {
            return AuditRecord::NotEnoughMoneyToWithdraw;
        }
        let Some(new_available) = self.available.try_change(-amount) else {
            // Technically this should never happen due to the check above
            return AuditRecord::MoneyUnderflow;
        };

        self.available = new_available;

        AuditRecord::Processed
    }

    /// Dispute previously deposited money
    pub fn dispute(&mut self, disputed_tx_id: TransactionId) -> AuditRecord {
        let Some(disputed_amount) = self.deposited_amounts.get(&disputed_tx_id) else {
            return AuditRecord::DisputedDepositNotFound;
        };
        let disputed_amount = *disputed_amount;

        let Some(new_held) = self.held.try_change(disputed_amount) else {
            return AuditRecord::MoneyOverflow;
        };

        let Some(new_available) = self.available.try_change(-disputed_amount) else {
            return AuditRecord::MoneyUnderflow;
        };

        self.held = new_held;
        self.available = new_available;
        self.disputed_amounts
            .insert(disputed_tx_id, disputed_amount);
        self.deposited_amounts.remove(&disputed_tx_id);

        AuditRecord::Processed
    }

    /// Resolve dispute
    pub fn resolve(&mut self, disputed_tx_id: TransactionId) -> AuditRecord {
        let Some(disputed_amount) = self.disputed_amounts.get(&disputed_tx_id) else {
            return AuditRecord::DisputeNotFound;
        };
        let disputed_amount = *disputed_amount;

        if self.held < disputed_amount {
            return AuditRecord::NotEnoughMoneyToRelease;
        }

        let Some(new_available) = self.available.try_change(disputed_amount) else {
            return AuditRecord::MoneyOverflow;
        };
        let Some(new_held) = self.held.try_change(-disputed_amount) else {
            return AuditRecord::MoneyUnderflow;
        };

        self.available = new_available;
        self.held = new_held;
        self.disputed_amounts.remove(&disputed_tx_id);
        self.deposited_amounts
            .insert(disputed_tx_id, disputed_amount);

        AuditRecord::Processed
    }

    pub fn chargeback(&mut self, disputed_tx_id: TransactionId) -> AuditRecord {
        let Some(disputed_amount) = self.disputed_amounts.get(&disputed_tx_id) else {
            return AuditRecord::DisputeNotFound;
        };
        let disputed_amount = *disputed_amount;

        if self.held < disputed_amount {
            return AuditRecord::NotEnoughMoneyToChargeBack;
        }

        let Some(new_held) = self.held.try_change(-disputed_amount) else {
            return AuditRecord::MoneyUnderflow;
        };

        self.held = new_held;
        self.disputed_amounts.remove(&disputed_tx_id);
        self.locked = true;

        AuditRecord::Processed
    }
}

/// Helper function to create accounts in tests
#[cfg(test)]
pub fn account(
    available: impl Into<MoneyAmount>,
    held: impl Into<MoneyAmount>,
    locked: bool,
) -> Account {
    Account {
        available: available.into(),
        held: held.into(),
        locked,
        deposited_amounts: Default::default(),
        disputed_amounts: Default::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::money;

    impl PartialEq for Account {
        fn eq(&self, other: &Self) -> bool {
            self.available == other.available
                && self.held == other.held
                && self.locked == other.locked
        }
    }

    #[test]
    fn total() {
        let account = account(100, 20, false);

        assert_eq!(account.total(), Some(MoneyAmount::from(120)));
    }

    #[test]
    fn total_fails_when_numbers_are_too_large() {
        let account = account(
            money::MAX.try_change(-10).unwrap(),
            MoneyAmount::from(20),
            false,
        );

        assert_eq!(account.total(), None);
    }
}
