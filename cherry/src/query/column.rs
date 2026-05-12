use crate::types::db::{DbType, DbValue};
use std::marker::PhantomData;
use std::ops::{BitAnd, BitOr};

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

/// Database column definitions
pub struct Column<T> {
    pub name: &'static str,
    _marker: PhantomData<T>,
}

impl<T> Column<T> {
    pub const fn new(name: &'static str) -> Self {
        Self { name, _marker: PhantomData }
    }

    pub fn eq(&self, value: impl Into<T>) -> Condition where T: DbType {
        Condition::Eq(self.name, value.into().to_db_value())
    }

    pub fn not_eq(&self, value: impl Into<T>) -> Condition where T: DbType {
        Condition::NotEq(self.name, value.into().to_db_value())
    }

    pub fn gt(&self, value: impl Into<T>) -> Condition where T: DbType {
        Condition::Gt(self.name, value.into().to_db_value())
    }

    pub fn is_null(&self) -> Condition {
        Condition::IsNull(self.name)
    }

    pub fn in_list(&self, values: Vec<impl Into<T>>) -> Condition where T: DbType {
        let db_values = values.into_iter().map(|v| v.into().to_db_value()).collect();
        Condition::In(self.name, db_values)
    }

    // Update assignment similarly, no longer pre-paste in advance.
    pub fn set(&self, value: impl Into<T>) -> Assignment where T: DbType {
        Assignment {
            column: self.name,
            value: value.into().to_db_value(),
        }
    }
}

pub struct Assignment {
    pub column: &'static str,
    pub value: DbValue,
}