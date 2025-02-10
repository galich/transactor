use crate::{
    account::{Account, AuditRecord, ClientId},
    transactions::{Transaction, TransactionDetail},
};
use std::collections::HashMap;

#[derive(Default)]
pub struct Processor {
    pub accounts: HashMap<ClientId, Account>,
}

impl Processor {
    /// Process transactions and return AuditRecord for each
    pub fn process<'a, T: IntoIterator<Item = &'a Transaction>>(
        &mut self,
        transactions: T,
    ) -> impl Iterator<Item = AuditRecord> + use<'_, 'a, T> {
        transactions
            .into_iter()
            .map(|transaction| self.process_transaction(transaction))
            .into_iter()
    }

    fn process_transaction(&mut self, tx: &Transaction) -> AuditRecord {
        let account = self
            .accounts
            .entry(tx.client_id)
            .or_insert_with(Default::default);

        match tx.detail {
            TransactionDetail::Deposit { amount } => account.deposit(tx.id, amount),
            TransactionDetail::Withdrawal { amount } => account.withdraw(amount),
            TransactionDetail::Dispute { tx_id } => account.dispute(tx_id),
            TransactionDetail::Resolve { tx_id } => account.resolve(tx_id),
            TransactionDetail::ChargeBack { tx_id } => account.chargeback(tx_id),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{
        account::{account, Account, AuditRecord},
        money::{self, MoneyAmount},
        processor::ClientId,
        transactions::{chargeback, deposit, dispute, resolve, withdraw},
    };
    use std::collections::HashMap;

    /// Assert that after processing of given transactions
    /// we get expected audit records and account states.
    fn assert_processing<'a, T: IntoIterator<Item = &'a Transaction>>(
        transactions: T,
        expected_audit: &[AuditRecord],
        expected_accounts: impl Into<HashMap<ClientId, Account>>,
    ) {
        let mut processor = Processor::default();
        let audit: Vec<AuditRecord> = processor.process(transactions).collect();
        let expected_accounts = expected_accounts.into();

        assert_eq!(audit, expected_audit);
        assert_eq!(processor.accounts, expected_accounts);
    }

    #[test]
    fn deposit_increase_available_in_correct_accounts() {
        assert_processing(
            &[deposit(2, 100, 99.8765), deposit(1, 101, 12.1234)],
            &[AuditRecord::Processed, AuditRecord::Processed],
            [
                (1, account(12.1234, 0, false)),
                (2, account(99.8765, 0, false)),
            ],
        );
    }

    #[test]
    fn deposit_fails_on_negative_amounts() {
        let money = MoneyAmount::from(10);
        assert_processing(
            &[deposit(1, 101, money), deposit(1, 101, -13)],
            &[AuditRecord::Processed, AuditRecord::CanNotDepositNegative],
            [(1, account(money, 0, false))],
        );
    }

    #[test]
    fn deposit_fails_on_overflow() {
        let large = money::MAX.try_change(-100).unwrap();
        assert_processing(
            &[deposit(1, 101, large), deposit(1, 101, 101)],
            &[AuditRecord::Processed, AuditRecord::MoneyOverflow],
            [(1, account(large, 0, false))],
        );
    }

    #[test]
    fn withdraw_decrease_amount() {
        assert_processing(
            &[deposit(1, 100, 12.1234), withdraw(1, 101, 2.12)],
            &[AuditRecord::Processed, AuditRecord::Processed],
            [(1, account(10.0034, 0, false))],
        );
    }

    #[test]
    fn withdraw_fails_on_negative_amount() {
        assert_processing(
            &[deposit(1, 100, 12.1234), withdraw(1, 101, -3)],
            &[AuditRecord::Processed, AuditRecord::CanNotWithdrawNegative],
            [(1, account(12.1234, 0, false))],
        );
    }

    #[test]
    fn withdraw_must_have_money() {
        assert_processing(
            &[deposit(1, 100, 12.1234), withdraw(1, 101, 20.12)],
            &[
                AuditRecord::Processed,
                AuditRecord::NotEnoughMoneyToWithdraw,
            ],
            [(1, account(12.1234, 0, false))],
        );
    }

    #[test]
    fn withdraw_fails_on_locked_account() {
        assert_processing(
            &[
                deposit(1, 100, 13),
                dispute(1, 100),
                chargeback(1, 100),
                withdraw(1, 104, 7),
            ],
            &[
                AuditRecord::Processed,
                AuditRecord::Processed,
                AuditRecord::Processed,
                AuditRecord::AccountLocked,
            ],
            [(1, account(0, 0, true))],
        );
    }

    #[test]
    fn dispute_deposits() {
        assert_processing(
            &[
                deposit(1, 100, 1000.0),
                deposit(1, 101, 200.0),
                dispute(1, 101),
            ],
            &[
                AuditRecord::Processed,
                AuditRecord::Processed,
                AuditRecord::Processed,
            ],
            [(1, account(1000, 200, false))],
        );
    }

    #[test]
    fn dispute_only_deposits() {
        assert_processing(
            &[
                deposit(1, 100, 1000.0),
                withdraw(1, 101, 200.0),
                dispute(1, 100),
                dispute(1, 101),
                dispute(1, 102),
            ],
            &[
                AuditRecord::Processed,
                AuditRecord::Processed,
                AuditRecord::Processed,
                AuditRecord::DisputedDepositNotFound,
                AuditRecord::DisputedDepositNotFound,
            ],
            [(1, account(-200, 1000, false))],
        );
    }

    #[test]
    fn dispute_only_once() {
        assert_processing(
            &[deposit(1, 100, 1000.0), dispute(1, 100), dispute(1, 100)],
            &[
                AuditRecord::Processed,
                AuditRecord::Processed,
                AuditRecord::DisputedDepositNotFound,
            ],
            [(1, account(0, 1000, false))],
        );
    }

    #[test]
    fn dispute_not_enough_funds() {
        assert_processing(
            &[
                deposit(1, 100, 600.0),
                withdraw(1, 101, 500.0),
                dispute(1, 100),
            ],
            &[
                AuditRecord::Processed,
                AuditRecord::Processed,
                AuditRecord::Processed,
            ],
            [(1, account(-500, 600, false))],
        );
    }

    #[test]
    fn resolve_decrease_held_funds() {
        assert_processing(
            &[deposit(1, 100, 1000.0), dispute(1, 100), resolve(1, 100)],
            &[
                AuditRecord::Processed,
                AuditRecord::Processed,
                AuditRecord::Processed,
            ],
            [(1, account(1000, 0, false))],
        );
    }

    #[test]
    fn chargeback_decrease_held_funds_and_freeze_account() {
        assert_processing(
            &[deposit(1, 100, 1000.0), dispute(1, 100), chargeback(1, 100)],
            &[
                AuditRecord::Processed,
                AuditRecord::Processed,
                AuditRecord::Processed,
            ],
            [(1, account(0, 0, true))],
        );
    }

    #[test]
    fn chargeback_once() {
        assert_processing(
            &[
                deposit(1, 100, 1000.0),
                dispute(1, 100),
                chargeback(1, 100),
                chargeback(1, 100),
            ],
            &[
                AuditRecord::Processed,
                AuditRecord::Processed,
                AuditRecord::Processed,
                AuditRecord::DisputeNotFound,
            ],
            [(1, account(0, 0, true))],
        );
    }
}
