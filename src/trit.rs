use std::fmt;

use crate::sum_result::SumResult;

/// In balanced ternary a "trit" is a three-value digit that can have
/// a value of -1, 0 or 1.
#[derive(Clone, Copy, Default, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum Trit {Neg, #[default]Zero, Pos}

impl fmt::Display for Trit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Trit::Neg => '-',
            Trit::Zero => '0',
            Trit::Pos => '+'
        })
    }
}

impl fmt::Debug for Trit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Opting to use the same output representation for Display
        // Debug traits.
        fmt::Display::fmt(&self, f)
    }
}

    impl From<char> for Trit {
    /// Convert the character representing of a trit into a Trit enum
    /// value. This representation accepts '+' as the +1 trit, '-' as the
    /// -1 trit and '0' as the zero trit. Any other characters will result
    /// in a panic.
    /// 
    /// * `encoded` A character representing a trit
    /// 
    /// **return** The trit represented by the submitted character, or the zero
    /// trit if an invalid character is provided.
    fn from(encoded: char) -> Self {
        match encoded {
            '-' => Trit::Neg,
            '0' => Trit::Zero,
            '+' => Trit::Pos,
            _ => panic!("Fail to parse invalid trit {}", encoded)
        }
    }
}

impl Trit {
    /// Return the opposite of the submitted trit, i.e. '+' is returned for
    /// '-' and vice versa. The negation of '0' is '0'.
    /// 
    /// * `trit`` A trit to find the negation for
    /// 
    /// **return** The negation of the submitted trit
    pub fn negate(self) -> Self {
        match self {
            Trit::Neg => Trit::Pos,
            Trit::Zero => Trit::Zero,
            Trit::Pos => Trit::Neg
        }
    }

    /// A half-adder that returns the sum of two trits. The result is both
    /// a direct value and potentially a carry trit that needs to be propagated
    /// to the next trit when summing a full ternary number. 
    /// 
    /// * `rhs` The other trit to add
    /// 
    /// **return** The result and carry for adding the two trits
    pub fn add(&self, rhs: &Trit) -> SumResult {
        match (self, rhs) {
            (l, Trit::Zero) => SumResult {result: *l, carry: Trit::Zero},
            (Trit::Zero, r) => SumResult {result: *r, carry: Trit::Zero},
            (l, r) if *l==*r => SumResult {result: l.negate(), carry: *l},
            (l, r) if *l==r.negate() => SumResult { result: Trit::Zero, carry: Trit::Zero },
            _ => unreachable!()
        }
    }

    /// A full-adder that sums three trits; usually matching-index trits from
    /// two ternary numbers and the carry from the previous index. The carry is
    /// not treated specially, but really is just a third trit to add. Returns
    /// a result and carry trit similar to the binary addTrits() case.
    /// 
    /// * `rhs` The other trit to add
    /// * `carry` A carry trit to also include in the addition
    /// 
    /// **return** The result and carry for adding the three trits
    pub fn add_with_carry(&self, rhs: &Trit, carry: &Trit) -> SumResult {
        match (self, rhs, carry) {
            // If any trit is zero we can reduce to the binary sum
            (Trit::Zero, r, c) => r.add(c),
            (l, Trit::Zero, c) => l.add(c),
            (l, r, Trit::Zero) => l.add(r),
            // If any two trits negate each other the remaining trit is the result
            (l, r, c) if l.negate()==*r => SumResult { result: *c, carry: Trit::Zero },
            (l, r, c) if l.negate()==*c => SumResult { result: *r, carry: Trit::Zero },
            (l, r, c) if r.negate()==*c => SumResult { result: *l, carry: Trit::Zero },
            // Else all three trits are the same, so the result is zero with a
            // carry trit
            (l, _, _) => SumResult { result: Trit::Zero, carry: *l }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pos_and_neg_cancel_out() {
        let result = Trit::Pos.add(&Trit::Neg);
        let expected = SumResult{result: Trit::Zero, carry: Trit::Zero};
        assert_eq!(result, expected);
    }
    
    #[test]
    fn add_two_pos_results_in_carry() {
        let result = Trit::Pos.add(&Trit::Pos);
        let expected = SumResult{result: Trit::Neg, carry: Trit::Pos};
        assert_eq!(result, expected);
    }
    
    #[test]
    fn add_two_neg_results_in_carry() {
        let result = Trit::Neg.add(&Trit::Neg);
        let expected = SumResult{result: Trit::Pos, carry: Trit::Neg};
        assert_eq!(result, expected);
    }

    #[test]
    fn pos_is_opposite_of_neg() {
        assert_eq!(Trit::Pos.negate(), Trit::Neg);
        assert_eq!(Trit::Neg.negate(), Trit::Pos);
    }
    
    #[test]
    fn zero_is_own_negative() {
        assert_eq!(Trit::Zero.negate(), Trit::Zero);
    }

    #[test]
    fn double_negation_has_no_change() {
        assert_eq!(Trit::Pos.negate().negate(), Trit::Pos);
        assert_eq!(Trit::Neg.negate().negate(), Trit::Neg);
    }
}