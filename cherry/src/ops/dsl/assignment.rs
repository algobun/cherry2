use crate::types::value::DbValue;

pub struct Assignment {
    pub column: &'static str,
    pub value: DbValue,
}