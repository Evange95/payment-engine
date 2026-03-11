use std::fmt;
use std::ops::{Add, Sub};
use std::str::FromStr;
use thiserror::Error;

/// Fixed-point amount with 4 decimal places.
/// 15000 represents 1.5000.
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
        let frac = abs % 10_000;
        if self.0 < 0 {
            write!(f, "-{whole}.{frac:04}")
        } else {
            write!(f, "{whole}.{frac:04}")
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
