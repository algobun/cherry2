use crate::{DbType, DbValue, Error, Result};
use jiff::Timestamp;

impl DbType for Timestamp {
    fn to_db_value(&self) -> DbValue {
        // jiff converts to a standard ISO 8601 string by default
        DbValue::Text(self.to_string())
    }

    fn from_db_value(value: DbValue) -> Result<Self> {
        match value {
            DbValue::Text(s) => {
                s.parse::<Timestamp>()
                    .map_err(|e| Error::ParseError(e.to_string()))
            }
            _ => Err(Error::TypeMismatch("Expected Text for jiff::Timestamp".into())),
        }
    }
}