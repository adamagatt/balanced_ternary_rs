use std::ops::{Add, Neg, Sub};
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

        Number::<N>::from_iter(trits)
    }

    fn from_iter(source: impl Iterator<Item = Trit>) -> Self {
        let mut source_with_idx= source.enumerate();

        // Populate lowest N trits with those provided from source
        let mut output = Number::<N>([(); N].map(|_| Trit::default()));
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
        number.0.iter().rev()
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
        Self(self.0
            .map(Trit::negate))
    }
}

impl <const N: usize> Add for Number<N> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let reverse_result_trits = self.0.iter().rev()
            .zip(rhs.0.iter().rev())
            .scan(Trit::ZERO, |carry, (lhs, rhs)| {
                let sum_result = lhs.add_with_carry(rhs, carry);
                *carry = sum_result.carry;
                Some(sum_result.result)
            });
        
        Number::<N>::from_iter(reverse_result_trits)
    }
}

impl <const N: usize> Sub for Number<N> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self + (-rhs)
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
    fn binary_operations() {
        let num_23 = Number::<8>::new("+0--");
        let num_33 = Number::<8>::new("++-0");

    assert_eq!(num_23 + num_33, Number::<8>::new("+-0+-")); // Sum to 56
    assert_eq!(num_23 - num_33, Number::<8>::new("-0-")); // Difference is -10
    // EXPECT_EQ(num_33 - num_23, BT::Number<8>{"+0+"}); // Difference is 10
    // EXPECT_EQ(num_23 * num_33, BT::Number<8>{"+00+0+0"}); // Product is 759
}
}