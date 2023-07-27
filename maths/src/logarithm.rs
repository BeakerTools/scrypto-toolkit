use radix_engine::types::{BnumI256, BnumU256, Decimal};
use crate::exponential::Exponential;

pub const LN_2: Decimal = Decimal(BnumI256::from_digits([693147180559945309, 0, 0, 0]));
pub const LN_10: Decimal = Decimal(BnumI256::from_digits([2302585092994045684, 0, 0, 0]));
// Next power of two for the BunmU256 representation of the Decimal 1
pub const NEXT_POWER_OF_TWO_FOR_ONE: BnumU256 = BnumU256::from_digits([1152921504606846976, 0, 0, 0]);

pub trait Logarithm {
    fn ln(self) -> Self;
    fn log2(self) -> Self;
    fn log10(self) -> Self;
    fn lob_base(self, base: Decimal) -> Self;
}

impl Logarithm for Decimal {
    /// Returns the natural logarithm of a [`Decimal`].
    ///
    /// The Taylor expansion of ln converges too slowly, so it is better to compute ln(y) using
    /// Halley’s method. It does it by computing the sequence x_n defined by induction:
    /// x_{n+1} = x_n + ( y - exp(x_n) )/( y + exp(x_n) ).
    /// Halley's method has a cubic convergence rate.
    fn ln(self) -> Self {
        assert!(self.is_positive(), "Logarithm is only defined for positive numbers");

        // If x < 1 we compute -ln(1/x) instead
        if self < Decimal::one()
        {
            -(Decimal::ONE / self).ln()
        }
        else {

            // Because, exp overflows very quickly, we rewrite y = 2^n(1 + x) with 0=< x <1.
            // This is possible because we make sure that y >= 1
            // Therefore, ln(y) = ln(1+x) + n*ln(2)
            let self_256 = BnumU256::try_from(self.0).unwrap();

            let pow_two = self_256.next_power_of_two()/NEXT_POWER_OF_TWO_FOR_ONE;
            let n = if pow_two == BnumU256::ONE { 0 } else { pow_two.0.ilog2() as u32 };

            let initial_value = self / Decimal::try_from(pow_two).unwrap();

            let mut result = initial_value;
            let mut last = result + Decimal::ONE;

            // Keep going while last and result are not equal ie. their difference is > 10^-18
            while last != result {
                last = result;
                let exp_last = last.exp();
                result = last + (initial_value - exp_last)/(initial_value + exp_last)*2;
            }

            result + Decimal::from(n)*LN_2
        }
    }

    /// Returns the binary logarithm of a [`Decimal`].
    fn log2(self) -> Self {
        self.ln()/LN_2
    }

    /// Returns the decimal logarithm of a [`Decimal`].
    fn log10(self) -> Self {
        self.ln()/LN_10
    }

    /// Returns the logarithm of a [`Decimal`] in a given base.
    fn lob_base(self, base: Decimal) -> Self {
        self.ln()/base.ln()
    }
}

#[cfg(test)]
mod test_ln{
    use radix_engine::types::{BnumI256, dec, Decimal};
    use crate::logarithm::{LN_2, Logarithm};
    use crate::{RELATIVE_PRECISION};
    use crate::exponential::Exponential;

    #[test]
    #[should_panic]
    fn test_ln_neg()
    {
        let _m = dec!(-5).ln();
    }

    #[test]
    #[should_panic]
    fn test_ln_zero()
    {
        let _m = Decimal::zero().ln();
    }

    #[test]
    fn test_ln_1()
    {
        assert!(Decimal::ONE.ln().abs() <= RELATIVE_PRECISION)
    }

    #[test]
    fn test_ln_0_5()
    {
        let rel_prec = (dec!("0.5").ln() + LN_2).abs()/LN_2;
        assert!(rel_prec < RELATIVE_PRECISION)
    }

    #[test]
    fn test_ln_smallest_dec()
    {
        let small = Decimal(BnumI256::ONE);
        let rel_prec = (small.ln() + dec!("41.446531673892822312")).abs()/dec!("41.446531673892822312");
        assert!(rel_prec < RELATIVE_PRECISION)
    }

    #[test]
    fn test_ln_12()
    {
        let rel_prec = (dec!(12).ln() - dec!("2.484906649788000310")).abs()/dec!("2.484906649788000310");
        assert!(rel_prec < RELATIVE_PRECISION)
    }

    #[test]
    fn test_ln_e()
    {
        let rel_prec = (Decimal::ONE.exp().ln() - Decimal::ONE).abs();
        assert!(rel_prec < RELATIVE_PRECISION);
    }

    #[test]
    fn test_ln_max()
    {
        let rel_prec = (dec!("135.305999368893231589") - Decimal::MAX.ln()).abs()/dec!("135.305999368893231589");
        assert!( rel_prec < RELATIVE_PRECISION);
    }

}