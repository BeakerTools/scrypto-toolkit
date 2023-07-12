use radix_engine::types::{BnumI256, BnumI384, Decimal};

pub const SMALLEST_NON_ZERO: Decimal = Decimal(BnumI256::from_digits([13893700547235832536, 18446744073709551613, 18446744073709551615, 18446744073709551615]));

pub trait Exponential {
    fn exp(self) -> Self;
}

impl Exponential for Decimal{

    /// Returns the exponential of a [`Decimal`] using Taylor series
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

    #[test]
    fn test_zero() {
        assert_eq!(Decimal::one(), Decimal::zero().exp());
    }

    #[test]
    fn test_one() {
        // True value is 2.718281828459045235 so we only have a small error
        assert_eq!(dec!("2.718281828459045226"), Decimal::one().exp())
    }

    #[test]
    fn test_neg_one() {
        // True value is 0.367879441171442321 so we only have a small error
        assert_eq!(dec!("0.367879441171442322"), (-Decimal::one()).exp())
    }

    #[test]
    fn test_smallest_non_zero() {
        assert_eq!(Decimal(BnumI256::ONE), SMALLEST_NON_ZERO.exp());
    }

    #[test]
    fn test_biggest_non_overflow() {
        // True value is 57896044618658097711785492504343953926634992332820282019728.792003956564819967
        // ie. Decimal::MAX
        assert_eq!(dec!("57896044618658095628311938129221453520480112551800193592119.589246893199839274"), dec!("135.305999368893231589").exp());
    }

    #[test]
    fn test_42() {
        // True value is 1739274941520501047.394681303611235226 so we only have a small error
        assert_eq!(dec!("1739274941520501037.39808957450998605"), dec!(42).exp());
    }

    #[test]
    fn test_100() {
        // True value is 26881171418161354484126255515800135873611118.773741922415191608 so we only have a small error
        assert_eq!(dec!("26881171418161353889139236717573277415827726.811691996064594573"), dec!(100).exp());
    }
}