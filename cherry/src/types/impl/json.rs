use crate::{DbType, DbValue, Error, Result};

impl DbType for serde_json::Value {
    fn to_db_value(&self) -> DbValue {
        // Serialize JSON objects as strings in SQLite
        DbValue::Text(self.to_string())
    }

    fn from_db_value(value: DbValue) -> Result<Self> {
        match value {
            DbValue::Text(s) => {
                serde_json::from_str(&s).map_err(|e| Error::ParseError(e.to_string()))
            }
            _ => Err(Error::TypeMismatch("Expected Text for JSON Value".into())),
        }
    }
}