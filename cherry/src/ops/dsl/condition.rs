use std::ops::{BitAnd, BitOr};
use crate::types::value::DbValue;

#[derive(Debug, Clone)]
pub enum Condition {
    Eq(&'static str, DbValue),
    NotEq(&'static str, DbValue),
    Gt(&'static str, DbValue),
    Gte(&'static str, DbValue),
    Lt(&'static str, DbValue),
    Lte(&'static str, DbValue),
    Like(&'static str, DbValue),
    In(&'static str, Vec<DbValue>),
    IsNull(&'static str),
    IsNotNull(&'static str),
    // Nested combination conditions
    And(Box<Condition>, Box<Condition>),
    Or(Box<Condition>, Box<Condition>),
}

// Gracefully supports `AND` and `OR` operators ( cond1 & cond2 )
impl BitAnd for Condition {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Condition::And(Box::new(self), Box::new(rhs))
    }
}

impl BitOr for Condition {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Condition::Or(Box::new(self), Box::new(rhs))
    }
}