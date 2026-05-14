use crate::{DbType, DbValue, Error, Result};
use uuid::Uuid;

impl DbType for Uuid {
    fn to_db_value(&self) -> DbValue {
        // Convert UUID to 16-byte array and save to blob
        DbValue::Blob(self.as_bytes().to_vec())
    }

    fn from_db_value(value: DbValue) -> Result<Self> {
        match value {
            DbValue::Blob(b) => {
                Uuid::from_slice(&b).map_err(|e| Error::ParseError(e.to_string()))
            }
            // Compatible with the situation where some users manually change the database to Text
            DbValue::Text(s) => {
                Uuid::parse_str(&s).map_err(|e| Error::ParseError(e.to_string()))
            }
            _ => Err(Error::TypeMismatch("Expected Blob or Text for Uuid".into())),
        }
    }
}