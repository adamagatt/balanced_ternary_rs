use std::fmt;

use crate::sum_result::SumResult;

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
    pub fn negate(self) -> Self {
        match self {
            Trit::Neg => Trit::Pos,
            Trit::Zero => Trit::Zero,
            Trit::Pos => Trit::Neg
        }
    }

    pub fn add(&self, rhs: &Trit) -> SumResult {
        match (self, rhs) {
            (l, Trit::Zero) => SumResult {result: *l, carry: Trit::Zero},
            (Trit::Zero, r) => SumResult {result: *r, carry: Trit::Zero},
            (l, r) if *l==*r => SumResult {result: l.negate(), carry: *l},
            (l, r) if *l==r.negate() => SumResult { result: Trit::Zero, carry: Trit::Zero },
            _ => unreachable!()
        }
    }

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