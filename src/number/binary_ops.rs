use std::iter::from_fn;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

use crate::number::Number;
use crate::sum_result::SumResult;
use crate::trit::Trit;

impl <const N: usize> Add for Number<N> {
    type Output = Self;

    /// Add this ternary number to another that has been provided. This
    /// may currently result in an overflow if the sum of the two numbers
    /// requires a length that is greater than the templated size N.
    /// 
    /// * `rhs` The number to add this number to
    /// 
    /// **returns** the result of adding this ternary number to the submitted
    /// number.
    fn add(self, rhs: Self) -> Self::Output {
        // Zip trits from both operands, going in reverse order so carries can propagate upwards
        let reverse_result_trits = self.0.iter().rev()
            .zip(rhs.0.iter().rev())
            // "Scan" as we need an output at each index, with accumulator propagating the carry trit
            .scan(Trit::Zero, |carry, (lhs, rhs)| {
                let SumResult{result, carry: new_carry} = lhs.add_with_carry(rhs, carry);
                *carry = new_carry;
                Some(result)
            });
        
        Number::<N>::from_rev_iter(reverse_result_trits)
    }
}

impl <const N: usize> AddAssign for Number<N> {
    /// In-place addition of another ternary number into this one, modifying
    /// this number rather than returning the sum. This may currently result
    /// in an overflow if the sum of the two numbers requires a length that
    /// is greater than the templated size N.
    /// 
    /// * `rhs` The number to add into this number
    fn add_assign(&mut self, rhs: Self) {
        // Much the same as the Add trait, but mutating self data in-place. As such we replace
        // "scan" with "for_each" to remove need for output. However we then lose the accumulator
        // So we need to declare an external `carry` variable.
        let mut carry = Trit::Zero;
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
    /// In-place addition of an individual trit. This may currently result
    /// in an overflow if the trit results in enough carries to reach the
    /// end of the templated size N.
    /// 
    /// * `rhs` The trit to add into this number
    fn add_assign(&mut self, rhs: Trit) {
        // Add the rhs to the least significant trit. Keep performing additions
        // and propagating carries through the indices until we don't need to
        // carry anymore or we run out of trit indices.
        let mut carry = rhs;
        for trit in self.0.iter_mut().rev() {
            if carry == Trit::Zero {break;}

            let SumResult{result, carry: new_carry} = trit.add(&carry);
            carry = new_carry;
            *trit = result;
        }
    }
}

impl <const N: usize> Sub for Number<N> {
    type Output = Self;

    /// Return the result of subtracting another ternary number from this one.
    /// This may currently result in an underflow if the result of the
    /// subtraction requires a length that is greater than the templated size
    /// N.
    /// 
    /// * `rhs` The number to subtract from this one
    /// 
    /// **returns** the result of subtracting the submitted ternary number from
    /// this one.
    fn sub(self, rhs: Self) -> Self::Output {
        self + -rhs
    }
}

impl <const N: usize> SubAssign for Number<N> {
    /// In-place subtraction of another ternary number from this one, modifying
    /// this number rather than returning the difference. This may currently
    /// result in an underflow if the difference of the two numbers requires a
    /// length that is greater than the templated size N.
    /// 
    /// * `rhs` The number to subtract from this number
    fn sub_assign(&mut self, rhs: Self) {
        *self += -rhs;
    }
}


impl <const N: usize> Mul for Number<N> {
    type Output = Self;

    /// Calculate the product of this ternary number multiplied with another
    /// that has been provided. This may currently result in an overflow or
    /// underflow if the product of the two numbers requires a length that is
    /// greater than the templated size N.
    /// 
    /// * `rhs` The number to multiply this number with
    /// 
    /// **returns** the product of this ternary number and the submitted number.
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
                    Trit::Neg => Some(-rhs_shifted),
                    Trit::Zero => None,
                    Trit::Pos => Some(rhs_shifted)
                }
            )
            .sum()
    }
}

impl <const N: usize> MulAssign for Number<N> {
    /// In-place multiplication of this ternary number with another that has
    /// been provided. This may currently result in an overflow or underflow
    /// if the product of the two numbers requires a length that is greater
    /// than the templated size N.
    /// 
    /// * `rhs` The number to multiply this number with
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

    /// Calculate the integer division of this ternary number by the supplied
    /// divisor, with the remainder discarded. This implementation rounds negative
    /// results towards zero rather than negative infinity, as the symmetry between
    /// positive and negative is a defining feature of balanced ternary.
    ///
    /// If the divisor is zero then the program will exit with an error mesage. In
    /// future integer division by zero will be handled gracefully with an error
    /// result or exception.
    /// 
    /// * `divisor` the number to integer divide this number by
    /// 
    /// **returns** the result of integer dividing this number by the supplied divisor
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
    /// In-place integer division of this ternary number with the supplied divisor,
    /// with the remainder discarded. This implementation rounds negative results
    /// towards zero rather than negative infinity, as the symmetry between
    /// positive and negative is a defining feature of balanced ternary.
    ///
    /// If the divisor is zero then the program will exit with an error mesage. In
    /// future integer division by zero will be handled gracefully with an error
    /// result or exception.
    /// 
    /// * `divisor` the number to integer divide this number by
    fn div_assign(&mut self, divisor: Self) {
        *self = *self / divisor;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binary_operations() {
        let num_23 = Number::<8>::from("+0--");
        let num_33 = Number::<8>::from("++-0");

        assert_eq!(num_23 + num_33, Number::<8>::from("+-0+-")); // Sum to 56
        assert_eq!(num_23 - num_33, Number::<8>::from("-0-")); // Difference is -10
        assert_eq!(num_33 - num_23, Number::<8>::from("+0+")); // Difference is 10
        assert_eq!(num_23 * num_33, Number::<8>::from("+00+0+0")); // Product is 759
    }

    #[test]
    fn in_place_binary_operations() {
        let num_23 = Number::<8>::from("+0--");
        let num_33 = Number::<8>::from("++-0");

        let mut temp = num_23;
        temp += num_33;
        assert_eq!(temp, Number::<8>::from("+-0+-")); // Sum to 56

        temp = num_23;
        temp -= num_33;
        assert_eq!(temp, Number::<8>::from("-0-")); // Difference is -10
        
        temp = num_33;
        temp -= num_23;
        assert_eq!(temp, Number::<8>::from("+0+")); // Difference is 10
        
        temp = num_23;
        temp *= num_33;
        assert_eq!(temp, Number::<8>::from("+00+0+0")); // Product is 759
}

    #[test]
    fn integer_division() {
        let num_59 = Number::<8>::from("+-+--");
        let num_60 = Number::<8>::from("+-+-0");
        let num_61 = Number::<8>::from("+-+-+");
        let num_12 = Number::<8>::from("++0");

        // Integral division with remainders discarded
        assert_eq!(num_59 / num_12, Number::<8>::from("0++")); // 59 / 12 = 4
        assert_eq!(num_60 / num_12, Number::<8>::from("+--")); // 60 / 12 = 5
        assert_eq!(num_61 / num_12, Number::<8>::from("+--")); // 61 / 12 = 5

        // Negatively signed numerators and divisors, results rounded towards zero
        assert_eq!(-num_59 /  num_12, Number::<8>::from("0--")); // -59 /  12 = -4
        assert_eq!( num_59 / -num_12, Number::<8>::from("0--")); //  59 / -12 = -4
        assert_eq!(-num_59 / -num_12, Number::<8>::from("0++")); // -59 / -12 =  4

        // Dividing zero by any number results in zero
        let num_0: Number<8> = Number::<8>::ZERO;

        assert_eq!(num_0 / num_60, num_0);    // 0 /  60 = 0
        assert_eq!(num_0 / (-num_60), num_0); // 0 / -60 = 0
    }

    #[test]
    #[should_panic(expected = "Attempt to divide by zero")]
    fn pos_divide_by_zero() {
        let num_61 = Number::<8>::from("+-+-+");
        let num_0: Number<8> = Number::<8>::ZERO;

        let _ = num_61 / num_0;
    }

    #[test]
    #[should_panic(expected = "Attempt to divide by zero")]
    fn neg_divide_by_zero() {
        let num_neg_61 = Number::<8>::from("-+-+-");
        let num_0: Number<8> = Number::<8>::ZERO;

        let _ = num_neg_61 / num_0;
    }  
}