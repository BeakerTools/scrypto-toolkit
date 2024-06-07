use crate::exponential::Exponential;
use crate::internal_prelude::*;
use crate::logarithm::Logarithm;

pub trait Power {
    fn pow(self, exp: Self) -> Self;
}

impl Power for Decimal {
    /// Returns number to the exp.
    fn pow(self, exp: Self) -> Self {
        (exp * self.ln()).exp()
    }
}
