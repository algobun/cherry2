use crate::Result;
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
use rusqlite::Result as RusqliteResult;
use std::str::from_utf8;
use crate::model::ColumnMeta;

pub trait Type: Sized {
    fn to_db(&self) -> Result<Value>;

    fn from_db(value: Value) -> Result<Self>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
}


impl ToSql for Value {
    fn to_sql(&self) -> RusqliteResult<ToSqlOutput<'_>> {
        match self {
            Value::Null => Ok(ToSqlOutput::from(rusqlite::types::Null)),
            Value::Integer(i) => Ok(ToSqlOutput::from(*i)),
            Value::Real(f) => Ok(ToSqlOutput::from(*f)),
            Value::Text(t) => Ok(ToSqlOutput::from(t.as_str())),
            Value::Blob(b) => Ok(ToSqlOutput::from(b.as_slice())),
        }
    }
}

impl FromSql for Value {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value {
            ValueRef::Null => Ok(Value::Null),
            ValueRef::Integer(i) => Ok(Value::Integer(i)),
            ValueRef::Real(f) => Ok(Value::Real(f)),
            ValueRef::Text(t) => {
                let s = from_utf8(t).map_err(|e| FromSqlError::Other(Box::new(e)))?;
                Ok(Value::Text(s.to_string()))
            }
            ValueRef::Blob(b) => Ok(Value::Blob(b.to_vec())),
        }
    }
}


#[derive(Debug, Clone)]
pub enum CondExpr {
    Column(ColumnMeta),
    Value(Value),
    Error(String),
}


impl From<ColumnMeta> for CondExpr {
    fn from(value: ColumnMeta) -> Self {
        CondExpr::Column(value)
    }
}

impl From<Result<Value>> for CondExpr {
    fn from(value: Result<Value>) -> Self {
        match value {
            Ok(v) => CondExpr::Value(v),
            Err(e) => CondExpr::Error(e.to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AssignExpr {
    Value(Value),
    Error(String),
}

impl From<Result<Value>> for AssignExpr {
    fn from(value: Result<Value>) -> Self {
        match value {
            Ok(v) => AssignExpr::Value(v),
            Err(e) => AssignExpr::Error(e.to_string()),
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

    impl Type for Role {
        fn to_db(&self) -> Result<Value> {
            match self {
                Role::Admin => Ok(Value::Text("admin".to_string())),
                Role::User => Ok(Value::Text("user".to_string())),
            }
        }

        fn from_db(value: Value) -> Result<Self> {
            match value {
                Value::Text(s) if s == "admin" => Ok(Role::Admin),
                Value::Text(s) if s == "user" => Ok(Role::User),
                _ => Err(Error::TypeMismatch("Invalid role mapping".to_string())),
            }
        }
    }
}