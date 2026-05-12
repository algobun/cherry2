use crate::Result;
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
use rusqlite::Result as RusqliteResult;
use std::str::from_utf8;

#[derive(Debug, Clone, PartialEq)]
pub enum DbValue {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
}

pub trait DbType: Sized {
    fn to_db_value(&self) -> DbValue;

    fn from_db_value(value: DbValue) -> Result<Self>;
}



impl ToSql for DbValue {
    fn to_sql(&self) -> RusqliteResult<ToSqlOutput<'_>> {
        match self {
            DbValue::Null => Ok(ToSqlOutput::from(rusqlite::types::Null)),
            DbValue::Integer(i) => Ok(ToSqlOutput::from(*i)),
            DbValue::Real(f) => Ok(ToSqlOutput::from(*f)),
            DbValue::Text(t) => Ok(ToSqlOutput::from(t.as_str())),
            DbValue::Blob(b) => Ok(ToSqlOutput::from(b.as_slice())),
        }
    }
}

impl FromSql for DbValue {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value {
            ValueRef::Null => Ok(DbValue::Null),
            ValueRef::Integer(i) => Ok(DbValue::Integer(i)),
            ValueRef::Real(f) => Ok(DbValue::Real(f)),
            ValueRef::Text(t) => {
                let s = from_utf8(t).map_err(|e| FromSqlError::Other(Box::new(e)))?;
                Ok(DbValue::Text(s.to_string()))
            }
            ValueRef::Blob(b) => Ok(DbValue::Blob(b.to_vec())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;

    enum Role {
        Admin,
        User,
    }

    impl DbType for Role {
        fn to_db_value(&self) -> DbValue {
            match self {
                Role::Admin => DbValue::Text("admin".to_string()),
                Role::User => DbValue::Text("user".to_string()),
            }
        }

        fn from_db_value(value: DbValue) -> Result<Self> {
            match value {
                DbValue::Text(s) if s == "admin" => Ok(Role::Admin),
                DbValue::Text(s) if s == "user" => Ok(Role::User),
                _ => Err(Error::TypeMismatch("Invalid role mapping".to_string())),
            }
        }
    }
}