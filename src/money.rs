use std::ops::Neg;

/// Integer type to be used for money amount with 4 decimal points.
///
/// i64 supports values up to 900 trillion.
/// (global world wealth estimated to be around 450 trillion in 2023).
///
/// Use i128 if more capacity is required.
pub(crate) type IntegerType = i64;

/// Fixed point money amount.
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct MoneyAmount(IntegerType);

impl MoneyAmount {
    /// Try to add/sub value to self
    /// Returns true if operation succeeeded.
    /// Returns false when adding a value overflows capacity or subtracting causes underflow.
    pub fn try_change(&self, value: impl Into<Self>) -> Option<Self> {
        let value = value.into().0;
        let result = if value > 0 {
            self.0.checked_add(value)
        } else {
            self.0.checked_sub(-value)
        };

        result.map(|result| MoneyAmount(result))
    }
}

impl From<f64> for MoneyAmount {
    fn from(value: f64) -> Self {
        Self((value * 10000.0).round() as IntegerType)
    }
}

impl From<IntegerType> for MoneyAmount {
    fn from(value: IntegerType) -> Self {
        Self((value * 10000).into())
    }
}

impl Neg for MoneyAmount {
    type Output = Self;

    fn neg(self) -> Self::Output {
        MoneyAmount(-self.0)
    }
}

impl std::fmt::Display for MoneyAmount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}{}.{:04}",
            if self.0 < 0 { "-" } else { "" },
            (self.0 / 10000).abs(),
            (self.0 % 10000).abs()
        ))
    }
}

impl PartialOrd for MoneyAmount {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl PartialEq<IntegerType> for MoneyAmount {
    fn eq(&self, other: &IntegerType) -> bool {
        self.0.eq(other)
    }
}

impl PartialOrd<IntegerType> for MoneyAmount {
    fn partial_cmp(&self, other: &IntegerType) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

#[cfg(test)]
pub static MAX: MoneyAmount = MoneyAmount(IntegerType::MAX);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_create_from_integer() {
        assert_eq!(MoneyAmount::from(123).0, 1230000);
    }

    #[test]
    fn can_create_from_float() {
        assert_eq!(MoneyAmount::from(123.4567).0, 1234567);
    }

    #[test]
    fn can_add_money() {
        assert_eq!(
            MoneyAmount::from(100.0).try_change(200),
            Some(MoneyAmount::from(300.0))
        );
    }

    #[test]
    fn detects_overflow() {
        let large = IntegerType::MAX - 100;
        assert!(MoneyAmount(large).try_change(200).is_none());
    }

    #[test]
    fn detects_underflow() {
        let small = IntegerType::MIN + 100;
        assert!(MoneyAmount(small).try_change(-200).is_none());
    }
}
