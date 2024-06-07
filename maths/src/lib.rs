use internal_prelude::*;
pub mod exponential;
pub(crate) mod internal_prelude;
pub mod logarithm;
pub mod power;

// Relative precision of the library is 10^-16
pub const RELATIVE_PRECISION: Decimal = Decimal(I192::from_digits([100, 0, 0]));
