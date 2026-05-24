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
                // Prioritize attempts at standard RFC 3339 (with T and Z)
                if let Ok(dt) = DateTime::parse_from_rfc3339(&s) {
                    return Ok(dt.with_timezone(&Utc));
                }

                // Tolerance degradation: try SQLite's native CURRENT_TIMESTAMP format (YYYY-MM-DD HH:MM:SS)
                // SQLite's CURRENT_TIMESTAMP is UTC time by default.
                if let Ok(ndt) = chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S") {
                    return Ok(DateTime::from_naive_utc_and_offset(ndt, Utc));
                }

                Err(Error::ParseError(format!("Invalid datetime format: {}", s)))

            }
            _ => Err(Error::TypeMismatch("Expected Text for DateTime".into())),
        }
    }
}