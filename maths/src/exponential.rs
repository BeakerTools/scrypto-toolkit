use radix_engine::types::{Decimal, I192, I256};

pub const SMALLEST_NON_ZERO: Decimal = Decimal(I192::from_digits([
    13893700547235832536,
    18446744073709551613,
    18446744073709551615,
]));

pub trait Exponential {
    fn exp(self) -> Self;
}

impl Exponential for Decimal {
    /// Returns the exponential of a [`Decimal`] using Taylor series.
    fn exp(self) -> Self {
        if self.is_zero() {
            Decimal::one()
        } else if self.is_negative() {
            if self < SMALLEST_NON_ZERO {
                Decimal::zero()
            } else {
                Decimal::one() / ((-self).exp())
            }
        } else {
            let self_384 = I256::from(self.0);
            let one_384 = I256::from(Decimal::ONE.0);
            let mut result = one_384;
            let mut added_term = self_384;
            let mut counter = I256::ONE;
            while added_term != I256::ZERO {
                result += added_term;
                counter += I256::ONE;
                added_term *= self_384 / counter;
                added_term /= one_384;
            }
            Decimal(I192::try_from(result).expect("Overflow"))
        }
    }
}

#[cfg(test)]
mod test_exp {
    use crate::exponential::{Exponential, SMALLEST_NON_ZERO};
    use crate::RELATIVE_PRECISION;
    use radix_engine::types::{dec, Decimal, I192};

    #[test]
    fn test_zero() {
        assert_eq!(Decimal::one(), Decimal::zero().exp());
    }

    #[test]
    fn test_one() {
        let rel_prec = (dec!("2.718281828459045235") - Decimal::one().exp())
            .checked_abs()
            .unwrap()
            / dec!("2.718281828459045235");
        assert!(rel_prec < RELATIVE_PRECISION);
    }

    #[test]
    fn test_neg_one() {
        let rel_prec = (dec!("0.367879441171442321") - (-Decimal::one()).exp())
            .checked_abs()
            .unwrap()
            / dec!("0.367879441171442321");
        assert!(rel_prec < RELATIVE_PRECISION);
    }

    #[test]
    fn test_smallest_non_zero() {
        assert_eq!(Decimal(I192::ONE), SMALLEST_NON_ZERO.exp());
    }

    #[test]
    fn test_biggest_non_overflow() {
        let true_val = Decimal::MAX;
        let rel_prec = (true_val - dec!("90.944579813056731786").exp())
            .checked_abs()
            .unwrap()
            / true_val;
        assert!(rel_prec < RELATIVE_PRECISION);
    }

    #[test]
    fn test_42() {
        let true_val = dec!("1739274941520501037.39808957450998605");
        let rel_prec = (true_val - dec!(42).exp()).checked_abs().unwrap() / true_val;
        assert!(rel_prec < RELATIVE_PRECISION);
    }

    #[test]
    fn test_57() {
        let true_val = dec!("5685719999335932222640348.820633253303372158");
        let rel_prec = (true_val - dec!(57).exp()).checked_abs().unwrap() / true_val;
        assert!(rel_prec < RELATIVE_PRECISION)
    }
}
