use radix_engine::types::Decimal;
use crate::exponential::Exponential;
use crate::logarithm::Logarithm;

pub trait Power {
    fn pow(self, exp: Self) -> Self;
}

impl Power for Decimal{

    /// Returns number to the exp.
    fn pow(self, exp: Self) -> Self {
        return (exp * self.ln()).exp();
    }
}