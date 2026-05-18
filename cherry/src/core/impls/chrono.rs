use crate::core::value::{Type, Value};
use crate::{Error, Result};
use chrono::{DateTime, Utc};

impl Type for DateTime<Utc> {
    fn to_db(&self) -> Result<Value> {
        // Save in standard RFC3339 text format, such as "2023-10-15T08:30: 00Z"
        Ok(Value::Text(self.to_rfc3339()))
    }

    fn from_db(value: Value) -> Result<Self> {
        match value {
            Value::Text(s) => {
                let dt = DateTime::parse_from_rfc3339(&s)
                    .map_err(|e| Error::ParseError(e.to_string()))?;
                Ok(dt.with_timezone(&Utc))
            }
            _ => Err(Error::TypeMismatch("Expected Text for DateTime".into())),
        }
    }
}