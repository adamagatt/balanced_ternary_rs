use std::ops::{Add, AddAssign, Neg, Shl, ShlAssign, Sub, SubAssign};
use std::{cmp::min, fmt};

use crate::trit::Trit;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Number<const N: usize> ([Trit; N]);

impl<const N: usize> Number<N> {
    fn new(encoded: &str) -> Self {
        let length = min(N, encoded.len());

        // View character slice as slice of trits, starting from right
        // hand size (lowest significant trit)
        let trits = encoded[..length].chars()
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
        let mut source_with_idx= source.enumerate();

        // Hack to populate N-length array with default values, as templated arrays cannot
        // current derive the Default trait, unfortunately.
        let mut output = Number::<N>([(); N].map(|_| Trit::default()));

        // Populate lowest N trits with those provided from source
        while let Some((idx, trit)) = source_with_idx.next() {
            // Early exit if more trits are provided than the size of the Number
            if idx == N {
                break;
            }

            output.0[N-1-idx] = trit;
        }
        output       
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
                Trit::ZERO => 0,
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
                let sum_result = lhs.add_with_carry(rhs, carry);
                *carry = sum_result.carry;
                Some(sum_result.result)
            });
        
        Number::<N>::from_rev_iter(reverse_result_trits)
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
                let sum_result = lhs.add_with_carry(rhs, &carry);
                carry = sum_result.carry;
                *lhs = sum_result.result;
            });
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
        let mut out = Number::<N>([Trit::ZERO; N]);

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
        let num_0 = Number::<8>::new("0");
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
    fn unary_negation() {
        let num_35 = Number::<8>::new("++0-");
        let num_0 = Number::<8>::new("0");

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
        // EXPECT_EQ(num_23 * num_33, BT::Number<8>{"+00+0+0"}); // Product is 759
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
        
        // temp = num_23;
        // temp *= num_33;
        // EXPECT_EQ(temp, BT::Number<8>{"+00+0+0"}); // Product is 759
}
}