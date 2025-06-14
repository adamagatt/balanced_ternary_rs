use std::fmt;

use crate::sum_result::SumResult;

#[derive(Debug, Clone, Copy, Default, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum Trit {NEG, #[default]ZERO, POS}

impl fmt::Display for Trit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Trit::NEG => '-',
            Trit::ZERO => '0',
            Trit::POS => '+'
        })
    }
}

impl From<char> for Trit {
    fn from(encoded: char) -> Self {
        match encoded {
            '-' => Trit::NEG,
            '0' => Trit::ZERO,
            '+' => Trit::POS,
            _ => panic!("Fail to parse invalid trit {}", encoded)
        }
    }
}

impl From<&char> for Trit {
    fn from(encoded: &char) -> Self {
        Trit::from(*encoded)
    }
}

impl Trit {
    pub fn negate(self) -> Self {
        match self {
            Trit::NEG => Trit::POS,
            Trit::ZERO => Trit::ZERO,
            Trit::POS => Trit::NEG
        }
    }

    pub fn add(&self, rhs: &Trit) -> SumResult {
        match (self, rhs) {
            (l, Trit::ZERO) => SumResult {result: *l, carry: Trit::ZERO},
            (Trit::ZERO, r) => SumResult {result: *r, carry: Trit::ZERO},
            (l, r) if *l==*r => SumResult {result: l.negate(), carry: *l},
            (l, r) if *l==r.negate() => SumResult { result: Trit::ZERO, carry: Trit::ZERO },
            _ => unreachable!()
        }
    }

    pub fn add_with_carry(&self, rhs: &Trit, carry: &Trit) -> SumResult {
        match (self, rhs, carry) {
            // If any trit is zero we can reduce to the binary sum
            (Trit::ZERO, r, c) => r.add(c),
            (l, Trit::ZERO, c) => l.add(c),
            (l, r, Trit::ZERO) => l.add(r),
            // If any two trits negate each other the remaining trit is the result
            (l, r, c) if l.negate()==*r => SumResult { result: *c, carry: Trit::ZERO },
            (l, r, c) if l.negate()==*c => SumResult { result: *r, carry: Trit::ZERO },
            (l, r, c) if r.negate()==*c => SumResult { result: *l, carry: Trit::ZERO },
            // Else all three trits are the same, so the result is zero with a
            // carry trit
            (l, _, _) => SumResult { result: Trit::ZERO, carry: *l }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pos_and_neg_cancel_out() {
        let result = Trit::POS.add(&Trit::NEG);
        let expected = SumResult{result: Trit::ZERO, carry: Trit::ZERO};
        assert_eq!(result, expected);
    }
    
    #[test]
    fn add_two_pos_results_in_carry() {
        let result = Trit::POS.add(&Trit::POS);
        let expected = SumResult{result: Trit::NEG, carry: Trit::POS};
        assert_eq!(result, expected);
    }
    
    #[test]
    fn add_two_neg_results_in_carry() {
        let result = Trit::NEG.add(&Trit::NEG);
        let expected = SumResult{result: Trit::POS, carry: Trit::NEG};
        assert_eq!(result, expected);
    }

    #[test]
    fn pos_is_opposite_of_neg() {
        assert_eq!(Trit::POS.negate(), Trit::NEG);
        assert_eq!(Trit::NEG.negate(), Trit::POS);
    }
    
    #[test]
    fn zero_is_own_negative() {
        assert_eq!(Trit::ZERO.negate(), Trit::ZERO);
    }

    #[test]
    fn double_negation_has_no_change() {
        assert_eq!(Trit::POS.negate().negate(), Trit::POS);
        assert_eq!(Trit::NEG.negate().negate(), Trit::NEG);
    }
}