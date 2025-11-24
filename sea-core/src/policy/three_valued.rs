//! Three-valued boolean logic (SQL-like UNKNOWN) for SEA policy evaluation.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ThreeValuedBool {
    True,
    False,
    Null,
}

impl ThreeValuedBool {
    pub fn and(self, other: Self) -> Self {
        use ThreeValuedBool::*;
        match (self, other) {
            (False, _) | (_, False) => False,
            (True, True) => True,
            (True, Null) | (Null, True) | (Null, Null) => Null,
        }
    }

    pub fn or(self, other: Self) -> Self {
        use ThreeValuedBool::*;
        match (self, other) {
            (True, _) | (_, True) => True,
            (False, False) => False,
            _ => Null,
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn not(self) -> Self {
        !self
    }

    pub fn implies(self, other: Self) -> Self {
        // A -> B == (not A) or B
        (!self).or(other)
    }

    pub fn from_option_bool(v: Option<bool>) -> Self {
        match v {
            Some(true) => ThreeValuedBool::True,
            Some(false) => ThreeValuedBool::False,
            None => ThreeValuedBool::Null,
        }
    }

    pub fn into_option(self) -> Option<bool> {
        match self {
            ThreeValuedBool::True => Some(true),
            ThreeValuedBool::False => Some(false),
            ThreeValuedBool::Null => None,
        }
    }

    pub fn fold_and<I: IntoIterator<Item = Self>>(iter: I) -> Self {
        let mut seen_null = false;
        for v in iter {
            match v {
                ThreeValuedBool::False => return ThreeValuedBool::False,
                ThreeValuedBool::Null => seen_null = true,
                ThreeValuedBool::True => {}
            }
        }
        if seen_null {
            ThreeValuedBool::Null
        } else {
            ThreeValuedBool::True
        }
    }

    pub fn fold_or<I: IntoIterator<Item = Self>>(iter: I) -> Self {
        let mut seen_null = false;
        for v in iter {
            match v {
                ThreeValuedBool::True => return ThreeValuedBool::True,
                ThreeValuedBool::Null => seen_null = true,
                ThreeValuedBool::False => {}
            }
        }
        if seen_null {
            ThreeValuedBool::Null
        } else {
            ThreeValuedBool::False
        }
    }
}

impl std::ops::Not for ThreeValuedBool {
    type Output = Self;

    fn not(self) -> Self::Output {
        use ThreeValuedBool::*;
        match self {
            True => False,
            False => True,
            Null => Null,
        }
    }
}

pub mod aggregators {
    #![allow(dead_code)]
    use rust_decimal::Decimal;

    /// Sum that returns None (Null) if any element is missing.
    pub fn sum_nullable(items: &[Option<Decimal>]) -> Option<Decimal> {
        let mut total = Decimal::ZERO;
        let mut any_null = false;
        for item in items {
            match item {
                Some(v) => total += *v,
                None => any_null = true,
            }
        }
        if any_null {
            None
        } else {
            Some(total)
        }
    }

    /// Sum that ignores NULLs.
    pub fn sum_nonnull(items: &[Option<Decimal>]) -> Decimal {
        let mut total = Decimal::ZERO;
        for v in items.iter().flatten() {
            total += *v;
        }
        total
    }

    /// Count all items, including nulls.
    pub fn count_all<T>(items: &[Option<T>]) -> usize {
        items.len()
    }

    /// Count only non-null items.
    pub fn count_nonnull<T>(items: &[Option<T>]) -> usize {
        items.iter().filter(|x| x.is_some()).count()
    }
}
