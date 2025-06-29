use std::fmt;

use crate::{number::Number, trit::Trit};

impl <const N: usize> From<&str> for Number<N> {
    fn from(encoded: &str) -> Self {
        // View character slice as slice of trits, starting from right
        // hand size (lowest significant trit)
        let trits = encoded.chars()
            .rev()
            .map(Trit::from);

        Number::<N>::from_rev_iter(trits)
    }
}

impl<const N: usize> From<Number<N>> for i32 {
    fn from(number: Number<N>) -> i32 {
        // Proceed through trits from lowest-order to highest
        number.0.iter().rev()
            // Add index since we will need it to determine value at that position
            .enumerate()
            .map(|(idx, trit)| match trit {
                Trit::Neg => -3_i32.pow(idx as u32),
                Trit::Zero => 0_i32,
                Trit::Pos => 3_i32.pow(idx as u32)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_representation() {
        let num_50 = Number::<8>::from("+-0--");
        
        assert_eq!(format!("{}", num_50), "000+-0-- (50)");
    }
}