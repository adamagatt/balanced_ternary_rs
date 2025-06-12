use crate::trit::Trit;

#[derive(Debug, PartialEq)]
pub struct SumResult {
    pub result: Trit,
    pub carry: Trit
}