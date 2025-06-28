use std::iter::{from_fn, Sum};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Shl, ShlAssign, Sub, SubAssign};
use std::fmt;

use crate::sum_result::SumResult;
use crate::trit::Trit;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Number<const N: usize> ([Trit; N]);

impl<const N: usize> Number<N> {
    const ZERO: Number<N> = Number::<N>([Trit::ZERO; N]);

    fn new(encoded: &str) -> Self {
        // View character slice as slice of trits, starting from right
        // hand size (lowest significant trit)
        let trits = encoded.chars()
            .rev()
            .map(Trit::from);

        Number::<N>::from_rev_iter(trits)
    }

    /// Builds a balanced ternary number of length N from the supplied iterator of trits. The
    /// iterator should be in reverse order to allow the number to be populated from least-
    /// to most-significant position. If more trits are provided than the size of the number
    /// then the excess will be lost; if fewer are provided then the higher-order trits will
    /// be padded with zeros.
    /// * `source` - An iterator that supplies Trits
    fn from_rev_iter(source: impl Iterator<Item = Trit>) -> Self {
        let mut output = Number::<N>::ZERO;

        // Populate lowest N trits with those provided from source
        for (idx, trit) in source.enumerate() {
            // Early exit if more trits are provided than the size of the Number
            if idx == N {
                break;
            }

            output.0[N-1-idx] = trit;
        }
        output       
    }
    
    fn inc(&mut self) {
        *self += Trit::POS;
    }

    fn dec(&mut self) {
        *self += Trit::NEG;
    }
}

impl<const N: usize> From<Number<N>> for i32 {
    fn from(number: Number<N>) -> i32 {
        // Proceed through trits from lowest-order to highest
        number.0.iter().rev()
            // Add index since we will need it to determine value at that position
            .enumerate()
            .map(|(idx, trit)| match trit {
                Trit::NEG => -3_i32.pow(idx as u32),
                Trit::ZERO => 0_i32,
                Trit::POS => 3_i32.pow(idx as u32)
            })
            .sum()
    }
}

impl<const N: usize> fmt::Display for Number<N> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..N {
            write!(f, "{}", self.0[i])?
        }
        write!(f, " ({})", i32::from(*self))
    }
}

impl<const N: usize> Neg for Number<N> {
    type Output = Self;
    
    fn neg(self) -> Self::Output {
        Self(self.0.map(Trit::negate))
    }
}

impl <const N: usize> Add for Number<N> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        // Zip trits from both operands, going in reverse order so carries can propagate upwards
        let reverse_result_trits = self.0.iter().rev()
            .zip(rhs.0.iter().rev())
            // "Scan" as we need an output at each index, with accumulator propagating the carry trit
            .scan(Trit::ZERO, |carry, (lhs, rhs)| {
                let SumResult{result, carry: new_carry} = lhs.add_with_carry(rhs, carry);
                *carry = new_carry;
                Some(result)
            });
        
        Number::<N>::from_rev_iter(reverse_result_trits)
    }
}

impl <const N: usize> Sum for Number<N> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Number::<N>::ZERO, std::ops::Add::add)
    }
}

impl <const N: usize> Sub for Number<N> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self + -rhs
    }
}

impl <const N: usize> AddAssign for Number<N> {
    fn add_assign(&mut self, rhs: Self) {
        // Much the same as the Add trait, but mutating self data in-place. As such we replace
        // "scan" with "for_each" to remove need for output. However we then lose the accumulator
        // So we need to declare an external `carry` variable.
        let mut carry = Trit::ZERO;
        self.0.iter_mut().rev()
            .zip(rhs.0.iter().rev())
            .for_each(|(lhs, rhs)| {
                let SumResult { result, carry: new_carry} = lhs.add_with_carry(rhs, &carry);
                carry = new_carry;
                *lhs = result;
            });
    }
}

impl <const N: usize> AddAssign<Trit> for Number<N> {
    fn add_assign(&mut self, rhs: Trit) {
        // Add the rhs to the least significant trit. Keep performing additions
        // and propagating carries through the indices until we don't need to
        // carry anymore or we run out of trit indices.
        let mut carry = rhs;
        for trit in self.0.iter_mut().rev() {
            if carry == Trit::ZERO {break;}

            let SumResult{result, carry: new_carry} = trit.add(&carry);
            carry = new_carry;
            *trit = result;
        }
    }
}

impl <const N: usize> SubAssign for Number<N> {
    fn sub_assign(&mut self, rhs: Self) {
        *self += -rhs;
    }
}

impl <const N: usize> Shl<usize> for Number<N> {
    type Output = Self;

    fn shl(self, positions: usize) -> Self::Output {
        let mut out = Number::<N>::ZERO;

        // Early exit if we left-shift far enough that our number just becomes zero
        if positions >= N {
            return out;
        }

        // Left shift is just copying the correct trits from our value to the
        // start of our zero-initialised output number 
        out.0[..(N-positions)].copy_from_slice(&self.0[positions..]);
        out
    }
}

impl <const N: usize> ShlAssign<usize> for Number<N> {
    fn shl_assign(&mut self, positions: usize) {
        // Early exit if we left-shift far enough that our number just becomes zero
        if positions >= N {
            self.0.fill(Trit::ZERO);
            return;
        }

        // An in-place left-shift is achieved by rotating our value array by the
        // specified number of positions and then zeroing out the least-significant
        // trits.
        self.0.rotate_left(positions);
        self.0[N-positions..].fill(Trit::ZERO);
    }
}

impl <const N: usize> Mul for Number<N> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        // Generator that will provide continually left-shifted copies
        // of the rhs operand. This will support the shift-and-add
        // approach of the multiplication.
        let mut rhs_shifted = rhs;
        let rhs_shifter = move || {
            let out = rhs_shifted;
            rhs_shifted <<= 1;
            Some(out)
        };

        self.0.iter().rev()
            .zip(from_fn(rhs_shifter))
            .filter_map(|(current_trit, rhs_shifted)| 
                match current_trit {
                    Trit::NEG => Some(-rhs_shifted),
                    Trit::ZERO => None,
                    Trit::POS => Some(rhs_shifted)
                }
            )
            .sum()
    }
}

impl <const N: usize> MulAssign for Number<N> {
    fn mul_assign(&mut self, rhs: Self) {
        // Based on the shift-and-add approach to multiplication I don't see an
        // obvious way to do a more efficient in-place multiplication operator.
        // We need a third spare variable to build the complete result and then
        // copy it into our own array, or otherwise we make a copy of our own 
        // array before zeroing it out and perform the shift-and-add in-place
        // on that array.

        *self = *self * rhs;
    }
}

impl <const N: usize> Div for Number<N> {
    type Output = Self;

    fn div(self, divisor: Self) -> Self::Output {
        if divisor == Number::<N>::ZERO {
            panic!("Attempt to divide by zero")
        }

        // Integer division implemented with a repeated subtraction approach. We
        // convert numerator and divisor to positive to perform the division, and
        // then decide whether to flip the result based on if they originally had
        // different signs.

        let numerator_is_negative = self < Number::<N>::ZERO;
        let mut abs_remainder = if numerator_is_negative {-self} else {self};

        let divisor_is_negative = divisor < Number::<N>::ZERO;
        let abs_divisor = if divisor_is_negative {-divisor} else {divisor};

        let mut quotient = Number::<N>::ZERO;
        while abs_remainder >= abs_divisor {
            abs_remainder -= abs_divisor;
            quotient.inc();
        }

        if numerator_is_negative ^ divisor_is_negative {
            -quotient
        } else {
            quotient
        }
    }
}

impl <const N: usize> DivAssign for Number<N> {
    fn div_assign(&mut self, divisor: Self) {
        *self = *self / divisor;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_representation() {
        let num_50 = Number::<8>::new("+-0--");
        
        assert_eq!(format!("{}", num_50), "000+-0-- (50)");
    }

    #[test]
    fn comparisons() {
        let num_0 = Number::<8>::ZERO;
        let num_17 = Number::<8>::new("+-0-");
        let num_17_copy = num_17;
        let num_neg_17 = Number::<8>::new("-+0+");

        assert_eq!(num_17, num_17_copy);

        assert_ne!(num_0, num_17);
        assert_ne!(num_17, num_neg_17);

        assert!(num_0 < num_17);
        assert!(num_neg_17 < num_0);
        assert!(num_neg_17 < num_17);

        assert!(num_17 > num_neg_17);
        assert!(num_0 > num_neg_17);
        assert!(num_17 > num_0);

        assert!(num_0 <= num_17);
        assert!(num_neg_17 <= num_0);
        assert!(num_neg_17 <= num_17);
        assert!(num_17 <= num_17_copy);

        assert!(num_17 >= num_neg_17);
        assert!(num_0 >= num_neg_17);
        assert!(num_17 >= num_0);
        assert!(num_17 >= num_17_copy);
    }

    #[test]
    fn increments() {
        let num_neg_one = Number::<8>::new("-");
        let num_0 = Number::<8>::ZERO;
        let num_one = Number::<8>::new("+");

        // Pre-increment provides the incremented value
        let mut temp = num_neg_one;
        temp.inc();
        assert_eq!(temp, num_0);
        
        temp = num_0;
        temp.inc();
        assert_eq!(temp, num_one);

        temp = num_neg_one;
        temp.inc();
        temp.inc();
        assert_eq!(temp, num_one);

        // Test a chain of carries
        let num_neg_14 = Number::<8>::new("-+++");
        temp = num_neg_14;
        temp.inc();
        assert_eq!(temp, Number::<8>::new("0---")); // -14 + 1 = -13
    }

    #[test]
    fn decrements() {
        let num_neg_one = Number::<8>::new("-");
        let num_0 = Number::<8>::ZERO;
        let num_one = Number::<8>::new("+");

        // Pre-decrement provides the decremented value
        let mut temp = num_0;
        temp.dec();
        assert_eq!(temp, num_neg_one);

        temp = num_one;
        temp.dec();
        assert_eq!(temp, num_0);

        temp = num_one;
        temp.dec();
        temp.dec();
        assert_eq!(temp, num_neg_one);

        // Test a chain of carries
        let num_14 = Number::<8>::new("+---");
        temp = num_14;
        temp.dec();
        assert_eq!(temp, Number::<8>::new("0+++")); // 14 - 1 = 13
    }

    #[test]
    fn unary_negation() {
        let num_35 = Number::<8>::new("++0-");
        let num_0 = Number::<8>::ZERO;

        assert_eq!(-num_35, Number::<8>::new("--0+")); // Negation is -35
        assert_eq!(-(-num_35), Number::<8>::new("++0-")); // Double negation is 35

        // Only one representation of zero, and so negative zero is still zero
        assert_eq!(-num_0, num_0);
    }

    #[test]
    fn left_shift() {
        let num_neg_8 = Number::<8>::new("-0+"); // -8

        assert_eq!(num_neg_8 << 1, Number::<8>::new("0000-0+0"));
        assert_eq!(num_neg_8 << 2, Number::<8>::new("000-0+00"));
        assert_eq!(num_neg_8 << 3, Number::<8>::new("00-0+000"));
        assert_eq!(num_neg_8 << 4, Number::<8>::new("0-0+0000"));
        assert_eq!(num_neg_8 << 5, Number::<8>::new("-0+00000"));
        assert_eq!(num_neg_8 << 6, Number::<8>::new("0+000000"));
        assert_eq!(num_neg_8 << 7, Number::<8>::new("+0000000"));
        assert_eq!(num_neg_8 << 8, Number::<8>::new("00000000"));
    }

    #[test]
    fn in_place_left_shift() {
        let mut shifting_num = Number::<8>::new("-0+"); // -8
        
        shifting_num <<= 1;
        assert_eq!(shifting_num, Number::<8>::new("0000-0+0"));
        shifting_num <<= 1;
        assert_eq!(shifting_num, Number::<8>::new("000-0+00"));
        shifting_num <<= 1;
        assert_eq!(shifting_num, Number::<8>::new("00-0+000"));
        shifting_num <<= 1;
        assert_eq!(shifting_num, Number::<8>::new("0-0+0000"));
        shifting_num <<= 1;
        assert_eq!(shifting_num, Number::<8>::new("-0+00000"));
        shifting_num <<= 1;
        assert_eq!(shifting_num, Number::<8>::new("0+000000"));
        shifting_num <<= 1;
        assert_eq!(shifting_num, Number::<8>::new("+0000000"));
        shifting_num <<= 1;
        assert_eq!(shifting_num, Number::<8>::new("00000000"));
    }

    #[test]
    fn binary_operations() {
        let num_23 = Number::<8>::new("+0--");
        let num_33 = Number::<8>::new("++-0");

        assert_eq!(num_23 + num_33, Number::<8>::new("+-0+-")); // Sum to 56
        assert_eq!(num_23 - num_33, Number::<8>::new("-0-")); // Difference is -10
        assert_eq!(num_33 - num_23, Number::<8>::new("+0+")); // Difference is 10
        assert_eq!(num_23 * num_33, Number::<8>::new("+00+0+0")); // Product is 759
    }

    #[test]
    fn in_place_binary_operations() {
        let num_23 = Number::<8>::new("+0--");
        let num_33 = Number::<8>::new("++-0");

        let mut temp = num_23;
        temp += num_33;
        assert_eq!(temp, Number::<8>::new("+-0+-")); // Sum to 56

        temp = num_23;
        temp -= num_33;
        assert_eq!(temp, Number::<8>::new("-0-")); // Difference is -10
        
        temp = num_33;
        temp -= num_23;
        assert_eq!(temp, Number::<8>::new("+0+")); // Difference is 10
        
        temp = num_23;
        temp *= num_33;
        assert_eq!(temp, Number::<8>::new("+00+0+0")); // Product is 759
}

    #[test]
    fn integer_division() {
        let num_59 = Number::<8>::new("+-+--");
        let num_60 = Number::<8>::new("+-+-0");
        let num_61 = Number::<8>::new("+-+-+");
        let num_12 = Number::<8>::new("++0");

        // Integral division with remainders discarded
        assert_eq!(num_59 / num_12, Number::<8>::new("0++")); // 59 / 12 = 4
        assert_eq!(num_60 / num_12, Number::<8>::new("+--")); // 60 / 12 = 5
        assert_eq!(num_61 / num_12, Number::<8>::new("+--")); // 61 / 12 = 5

        // Negatively signed numerators and divisors, results rounded towards zero
        assert_eq!(-num_59 /  num_12, Number::<8>::new("0--")); // -59 /  12 = -4
        assert_eq!( num_59 / -num_12, Number::<8>::new("0--")); //  59 / -12 = -4
        assert_eq!(-num_59 / -num_12, Number::<8>::new("0++")); // -59 / -12 =  4

        // Dividing zero by any number results in zero
        let num_0: Number<8> = Number::<8>::ZERO;

        assert_eq!(num_0 / num_60, num_0);    // 0 /  60 = 0
        assert_eq!(num_0 / (-num_60), num_0); // 0 / -60 = 0
    }

    #[test]
    #[should_panic(expected = "Attempt to divide by zero")]
    fn pos_divide_by_zero() {
        let num_61 = Number::<8>::new("+-+-+");
        let num_0: Number<8> = Number::<8>::ZERO;

        let _ = num_61 / num_0;
    }

    #[test]
    #[should_panic(expected = "Attempt to divide by zero")]
    fn neg_divide_by_zero() {
        let num_neg_61 = Number::<8>::new("-+-+-");
        let num_0: Number<8> = Number::<8>::ZERO;

        let _ = num_neg_61 / num_0;
    }  

}