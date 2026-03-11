use std::fmt;
use std::ops::{Add, Sub};
use std::str::FromStr;
use thiserror::Error;

/// Fixed-point amount with 4 decimal places of internal precision.
/// 15000 represents 1.5000. Displayed with 2 decimal places (rounded).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Amount(i64);

impl Amount {
    pub const ZERO: Self = Self(0);

    pub fn is_negative(&self) -> bool {
        self.0 < 0
    }
}

impl FromStr for Amount {
    type Err = ParseAmountError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(ParseAmountError::Empty);
        }

        let (whole, frac) = match s.split_once('.') {
            Some((w, f)) => (w, f),
            None => (s, ""),
        };

        let negative = whole.starts_with('-');
        let whole_abs = if negative { &whole[1..] } else { whole };

        let whole_val: i64 = if whole_abs.is_empty() {
            0
        } else {
            whole_abs
                .parse()
                .map_err(|_| ParseAmountError::Invalid(s.to_string()))?
        };

        let frac_val: i64 = if frac.is_empty() {
            0
        } else if frac.len() > 4 {
            return Err(ParseAmountError::TooManyDecimals);
        } else {
            let padded = format!("{frac:0<4}");
            padded
                .parse()
                .map_err(|_| ParseAmountError::Invalid(s.to_string()))?
        };

        let raw = whole_val * 10_000 + frac_val;
        Ok(Self(if negative { -raw } else { raw }))
    }
}

impl Add for Amount {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl Sub for Amount {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let abs = self.0.unsigned_abs();
        let whole = abs / 10_000;
        let frac = (abs % 10_000 + 50) / 100; // round to 2 decimal places
        if frac >= 100 {
            // rounding overflowed (e.g. 0.9999 -> 1.00)
            let whole = whole + 1;
            if self.0 < 0 {
                write!(f, "-{whole}.00")
            } else {
                write!(f, "{whole}.00")
            }
        } else if self.0 < 0 {
            write!(f, "-{whole}.{frac:02}")
        } else {
            write!(f, "{whole}.{frac:02}")
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ParseAmountError {
    #[error("empty amount string")]
    Empty,
    #[error("invalid amount: {0}")]
    Invalid(String),
    #[error("more than 4 decimal places")]
    TooManyDecimals,
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Parsing tests ---

    #[test]
    fn parses_whole_number() {
        let amount: Amount = "42".parse().unwrap();
        assert_eq!(amount, Amount(420_000));
    }

    #[test]
    fn parses_with_decimal() {
        let amount: Amount = "1.5".parse().unwrap();
        assert_eq!(amount, Amount(15_000));
    }

    #[test]
    fn parses_four_decimal_places() {
        let amount: Amount = "1.2345".parse().unwrap();
        assert_eq!(amount, Amount(12_345));
    }

    #[test]
    fn parses_negative() {
        let amount: Amount = "-3.50".parse().unwrap();
        assert_eq!(amount, Amount(-35_000));
    }

    #[test]
    fn parses_with_leading_whitespace() {
        let amount: Amount = "  7.25  ".parse().unwrap();
        assert_eq!(amount, Amount(72_500));
    }

    #[test]
    fn rejects_empty_string() {
        assert_eq!("".parse::<Amount>(), Err(ParseAmountError::Empty));
    }

    #[test]
    fn rejects_too_many_decimals() {
        assert_eq!(
            "1.23456".parse::<Amount>(),
            Err(ParseAmountError::TooManyDecimals)
        );
    }

    #[test]
    fn rejects_invalid_input() {
        assert!("abc".parse::<Amount>().is_err());
    }

    // --- Display tests ---

    #[test]
    fn displays_zero() {
        assert_eq!(Amount::ZERO.to_string(), "0.00");
    }

    #[test]
    fn displays_whole_number_with_two_decimals() {
        let amount: Amount = "2".parse().unwrap();
        assert_eq!(amount.to_string(), "2.00");
    }

    #[test]
    fn displays_fractional_rounded_to_two_decimals() {
        let amount: Amount = "1.5".parse().unwrap();
        assert_eq!(amount.to_string(), "1.50");
    }

    #[test]
    fn displays_rounding_up() {
        let amount: Amount = "1.005".parse().unwrap();
        assert_eq!(amount.to_string(), "1.01");
    }

    #[test]
    fn displays_rounding_down() {
        let amount: Amount = "1.004".parse().unwrap();
        assert_eq!(amount.to_string(), "1.00");
    }

    #[test]
    fn displays_rounding_overflow() {
        let amount: Amount = "0.9999".parse().unwrap();
        assert_eq!(amount.to_string(), "1.00");
    }

    #[test]
    fn displays_negative() {
        let amount: Amount = "-3.50".parse().unwrap();
        assert_eq!(amount.to_string(), "-3.50");
    }

    #[test]
    fn displays_negative_rounding_overflow() {
        let amount: Amount = "-0.9999".parse().unwrap();
        assert_eq!(amount.to_string(), "-1.00");
    }
}
