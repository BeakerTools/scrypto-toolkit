use radix_engine::types::{Decimal, I192};

pub mod exponential;
pub mod logarithm;
pub mod power;

// Relative precision of the library is 10^-16
pub const RELATIVE_PRECISION: Decimal = Decimal(I192::from_digits([100, 0, 0]));
