mod conversions;
mod binary_ops;

use std::iter::Sum;
use std::ops::{Neg, Shl, ShlAssign};

use crate::trit::Trit;

/// A number in ternary is an array of trit values, similar to how a binary
/// encoding is an array of bits. This is an implementation of a number in
/// "Balanced Ternary", a system where each trit can be "-1", "0" or "+1".
/// 
/// This is a templated class to allow the user to specify the number of trits
/// to use for the number. All binary operations only support operating on
/// numbers that share the same size.
/// 
/// * `N` The number of trits to use in the number.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Number<const N: usize> ([Trit; N]);

impl<const N: usize> Number<N> {
    /// A balanced ternary representing zero by having all trits
    /// set to their zero values.
    const ZERO: Number<N> = Number::<N>([Trit::Zero; N]);

    /// Builds a balanced ternary number of length N from the supplied iterator of trits. The
    /// iterator should be in reverse order to allow the number to be populated from least-
    /// to most-significant position. If more trits are provided than the size of the number
    /// then the excess will be lost; if fewer are provided then the higher-order trits will
    /// be padded with zeros.
    /// 
    /// * `source` - An iterator that supplies Trits
    /// 
    /// **returns** A balanced ternary number from the supplied trits
    pub fn from_rev_iter(source: impl Iterator<Item = Trit>) -> Self {
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
    
    /// Increments the number by adding 1. This may result in a positive
    /// wraparound if all trits are already positive.
    pub fn inc(&mut self) {
        *self += Trit::Pos;
    }

    /// Decrements the number by adding 1. This may result in a negative
    /// wraparound if all trits are already negative.
    pub fn dec(&mut self) {
        *self += Trit::Neg;
    }
}

impl <const N: usize> Neg for Number<N> {
    type Output = Self;
    
    /// Unary negation of the ternary number, where every trit simply has
    /// its value flipped.
    /// 
    /// **return** The unary negation of this ternary number
    fn neg(self) -> Self::Output {
        Self(self.0.map(Trit::negate))
    }
}

impl <const N: usize> Sum for Number<N> {
    /// Implementing this trait to allow an iterator of numbers to
    /// be summed together. No special logic, simply the usual fold
    /// over the addition operator. But apparently needs to be explicitly
    /// implemented as no derive macro appears to be available.
    /// 
    /// * `iter` An iterator producing numbers
    /// 
    /// **returns** a number representing the sum of all the supplied numbers
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Number::<N>::ZERO, std::ops::Add::add)
    }
}

impl <const N: usize> Shl<usize> for Number<N> {
    type Output = Self;

    /// Return the result of left-shifting this number by a specified amount
    /// of trit positions. As each trit is explicitly signed this operation
    /// is always a signed shift. This has the usual effect of multiplying the
    /// number by 3. An over- or under-flow can occur if a non-zero most
    /// significant trit is shifted, and this can potentially change the sign
    /// of the result.
    /// 
    /// * `positions` The amount of trits to shift the number by
    /// 
    /// **returns** The result of left-shifting this number by the specified number
    /// of trit positions.
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
    /// In-place left-shift operation of this number by a specified amount
    /// of trit positions. As each trit is explicitly signed this operation
    /// is always a signed shift. This has the usual effect of multiplying the
    /// number by 3. An over- or under-flow can occur if a non-zero most
    /// significant trit is shifted, and this can potentially change the sign
    /// of this number.
    ///  
    /// * `positions` The amount of trits to shift this number by
    fn shl_assign(&mut self, positions: usize) {
        // Early exit if we left-shift far enough that our number just becomes zero
        if positions >= N {
            self.0.fill(Trit::Zero);
            return;
        }

        // An in-place left-shift is achieved by rotating our value array by the
        // specified number of positions and then zeroing out the least-significant
        // trits.
        self.0.rotate_left(positions);
        self.0[N-positions..].fill(Trit::Zero);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comparisons() {
        let num_0 = Number::<8>::ZERO;
        let num_17 = Number::<8>::from("+-0-");
        let num_17_copy = num_17;
        let num_neg_17 = Number::<8>::from("-+0+");

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
        let num_neg_one = Number::<8>::from("-");
        let num_0 = Number::<8>::ZERO;
        let num_one = Number::<8>::from("+");

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
        let num_neg_14 = Number::<8>::from("-+++");
        temp = num_neg_14;
        temp.inc();
        assert_eq!(temp, Number::<8>::from("0---")); // -14 + 1 = -13
    }

    #[test]
    fn decrements() {
        let num_neg_one = Number::<8>::from("-");
        let num_0 = Number::<8>::ZERO;
        let num_one = Number::<8>::from("+");

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
        let num_14 = Number::<8>::from("+---");
        temp = num_14;
        temp.dec();
        assert_eq!(temp, Number::<8>::from("0+++")); // 14 - 1 = 13
    }

    #[test]
    fn unary_negation() {
        let num_35 = Number::<8>::from("++0-");
        let num_0 = Number::<8>::ZERO;

        assert_eq!(-num_35, Number::<8>::from("--0+")); // Negation is -35
        assert_eq!(-(-num_35), Number::<8>::from("++0-")); // Double negation is 35

        // Only one representation of zero, and so negative zero is still zero
        assert_eq!(-num_0, num_0);
    }

    #[test]
    fn left_shift() {
        let num_neg_8 = Number::<8>::from("-0+"); // -8

        assert_eq!(num_neg_8 << 1, Number::<8>::from("0000-0+0"));
        assert_eq!(num_neg_8 << 2, Number::<8>::from("000-0+00"));
        assert_eq!(num_neg_8 << 3, Number::<8>::from("00-0+000"));
        assert_eq!(num_neg_8 << 4, Number::<8>::from("0-0+0000"));
        assert_eq!(num_neg_8 << 5, Number::<8>::from("-0+00000"));
        assert_eq!(num_neg_8 << 6, Number::<8>::from("0+000000"));
        assert_eq!(num_neg_8 << 7, Number::<8>::from("+0000000"));
        assert_eq!(num_neg_8 << 8, Number::<8>::from("00000000"));
    }

    #[test]
    fn in_place_left_shift() {
        let mut shifting_num = Number::<8>::from("-0+"); // -8
        
        shifting_num <<= 1;
        assert_eq!(shifting_num, Number::<8>::from("0000-0+0"));
        shifting_num <<= 1;
        assert_eq!(shifting_num, Number::<8>::from("000-0+00"));
        shifting_num <<= 1;
        assert_eq!(shifting_num, Number::<8>::from("00-0+000"));
        shifting_num <<= 1;
        assert_eq!(shifting_num, Number::<8>::from("0-0+0000"));
        shifting_num <<= 1;
        assert_eq!(shifting_num, Number::<8>::from("-0+00000"));
        shifting_num <<= 1;
        assert_eq!(shifting_num, Number::<8>::from("0+000000"));
        shifting_num <<= 1;
        assert_eq!(shifting_num, Number::<8>::from("+0000000"));
        shifting_num <<= 1;
        assert_eq!(shifting_num, Number::<8>::from("00000000"));
    }
}