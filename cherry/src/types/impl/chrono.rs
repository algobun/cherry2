use crate::{DbType, DbValue, Error, Result};
use chrono::{DateTime, Utc};

impl DbType for DateTime<Utc> {
    fn to_db_value(&self) -> DbValue {
        // 存为标准的 RFC3339 文本格式，例如 "2023-10-15T08:30:00Z"
        DbValue::Text(self.to_rfc3339())
    }

    fn from_db_value(value: DbValue) -> Result<Self> {
        match value {
            DbValue::Text(s) => {
                let dt = DateTime::parse_from_rfc3339(&s)
                    .map_err(|e| Error::ParseError(e.to_string()))?;
                Ok(dt.with_timezone(&chrono::Utc))
            }
            _ => Err(Error::TypeMismatch("Expected Text for DateTime".into())),
        }
    }
}