use crate::exponential::Exponential;
use crate::logarithm::Logarithm;
use radix_engine::types::Decimal;

pub trait Power {
    fn pow(self, exp: Self) -> Self;
}

impl Power for Decimal {
    /// Returns number to the exp.
    fn pow(self, exp: Self) -> Self {
        (exp * self.ln()).exp()
    }
}
