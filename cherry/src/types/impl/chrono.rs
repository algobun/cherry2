use crate::{Error, Result, Type, Value};
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
                Ok(dt.with_timezone(&chrono::Utc))
            }
            _ => Err(Error::TypeMismatch("Expected Text for DateTime".into())),
        }
    }
}