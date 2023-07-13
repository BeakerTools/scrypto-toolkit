use radix_engine::types::{BnumI256, Decimal};
use crate::exponential::Exponential;

pub const EULER_CONST: Decimal = Decimal(BnumI256::from_digits([2718281828459045226, 0, 0, 0]));

pub trait Logarithm {
    fn ln(self) -> Self;
}

impl Logarithm for Decimal {
    /// Returns the logarithm of a [`Decimal`].
    ///
    /// The Taylor expansion of ln converges too slowly, so it is better to compute ln(y) using
    /// Halley’s method. It does it by computing the sequence x_n defined by induction:
    /// x_{n+1} = x_n + ( y - exp(x_n) )/( y + exp(x_n) ).
    /// Halley's method has a cubic convergence rate.
    fn ln(self) -> Self {
        assert!(self.is_positive(), "Logarithm is only defined for positive numbers");

        // Because, exp overflows very quickly, we rewrite y = a*e^n with 0<a<=1
        // Therefore, ln(y) = ln(a) + n
        let mut n = 0;
        let mut value = self.clone();
        while value > EULER_CONST {
            value = value / EULER_CONST;
            n +=1 ;
        }
        // Start with an arbitrary number as the first guess
        let mut result = value / Decimal::from(2u8);

        // Too small to represent, so we start with self
        // Future iterations could actually avoid using a decimal altogether and use a buffered
        // vector, only combining back into a decimal on return
        if result.is_zero() {
            result = value;
        }
        let mut last = result + Decimal::ONE;

        // Keep going while last and result are not equal ie. their difference is > 10^-18
        while last != result {
            last = result;
            let exp_last = last.exp();
            result = last + (value - exp_last)/(value + exp_last)*2;
        }

        result + Decimal::from(n)
    }
}

#[cfg(test)]
mod test_ln{
    use radix_engine::types::{dec, Decimal};
    use crate::logarithm::{EULER_CONST, Logarithm};
    use crate::{RELATIVE_PRECISION};

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
    fn test_ln_12()
    {
        let rel_prec = (dec!(12).ln() - dec!("2.484906649788000310")).abs()/dec!("2.484906649788000310");
        assert!(rel_prec < RELATIVE_PRECISION)
    }

    #[test]
    fn test_ln_e()
    {
        let rel_prec = (EULER_CONST.ln() - Decimal::one()).abs();
        assert!(rel_prec < RELATIVE_PRECISION);
    }

    #[test]
    fn test_ln_max()
    {
        let rel_prec = (dec!("135.305999368893231589") - Decimal::MAX.ln()).abs()/dec!("135.305999368893231589");
        assert!( rel_prec < RELATIVE_PRECISION);
    }

}