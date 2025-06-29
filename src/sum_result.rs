use crate::trit::Trit;

/// When calculating a sum we need to know the in-place result and the
/// carry, which is also a trit.
#[derive(Debug, PartialEq)]
pub struct SumResult {
    pub result: Trit,
    pub carry: Trit
}