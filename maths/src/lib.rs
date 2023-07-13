use radix_engine::types::{BnumI256, Decimal};

pub mod exponential;
pub mod logarithm;
pub mod power;

// Relative precision of the library is 10^-16
pub const RELATIVE_PRECISION: Decimal = Decimal(BnumI256::from_digits([100, 0, 0, 0]));