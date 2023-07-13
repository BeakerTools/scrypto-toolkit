use radix_engine::types::{BnumI256, BnumI384, Decimal};

pub const SMALLEST_NON_ZERO: Decimal = Decimal(BnumI256::from_digits([13893700547235832536, 18446744073709551613, 18446744073709551615, 18446744073709551615]));

pub trait Exponential {
    fn exp(self) -> Self;
}

impl Exponential for Decimal{

    /// Returns the exponential of a [`Decimal`] using Taylor series.
    fn exp(self) -> Self {
        return if self.is_zero(){
            Decimal::one()
        } else if self.is_negative() {
            if self < SMALLEST_NON_ZERO {
                Decimal::zero()
            }
            else {
                Decimal::one() / ((-self).exp())
            }
        } else {
            let self_384 = BnumI384::from(self.0);
            let one_384 = BnumI384::from(Decimal::ONE.0);
            let mut result = one_384;
            let mut added_term = self_384.clone();
            let mut counter = BnumI384::ONE;
            while added_term != BnumI384::ZERO {
                result += added_term;
                counter += BnumI384::ONE;
                added_term = added_term * (self_384 / counter);
                added_term /= one_384;
            }
            Decimal(BnumI256::try_from(result).expect("Overflow"))
        }
    }
}

#[cfg(test)]
mod test_exp{
    use radix_engine::types::{BnumI256, dec, Decimal};
    use crate::exponential::{Exponential, SMALLEST_NON_ZERO};
    use crate::RELATIVE_PRECISION;

    #[test]
    fn test_zero() {
        assert_eq!(Decimal::one(), Decimal::zero().exp());
    }

    #[test]
    fn test_one() {
        let rel_prec = (dec!("2.718281828459045235") - Decimal::one().exp()).abs()/dec!("2.718281828459045235");
        assert!(rel_prec < RELATIVE_PRECISION);
    }

    #[test]
    fn test_neg_one() {
        let rel_prec = (dec!("0.367879441171442321") - (-Decimal::one()).exp()).abs()/dec!("0.367879441171442321");
        assert!( rel_prec < RELATIVE_PRECISION);
    }

    #[test]
    fn test_smallest_non_zero() {
        assert_eq!(Decimal(BnumI256::ONE), SMALLEST_NON_ZERO.exp());
    }

    #[test]
    fn test_biggest_non_overflow() {
        let true_val = dec!("57896044618658097711785492504343953926634992332820282019728.792003956564819967");
        let rel_prec = (true_val - dec!("135.305999368893231589").exp()).abs()/ true_val;
        assert!(rel_prec < RELATIVE_PRECISION);
    }

    #[test]
    fn test_42() {
        let true_val = dec!("1739274941520501037.39808957450998605");
        let rel_prec = (true_val - dec!(42).exp()).abs()/true_val;
        assert!(rel_prec < RELATIVE_PRECISION);
    }

    #[test]
    fn test_100() {
        let true_val = dec!("26881171418161354484126255515800135873611118.773741922415191608");
        let rel_prec = (true_val - dec!(100).exp()).abs()/true_val;
        assert!(rel_prec < RELATIVE_PRECISION)
    }
}