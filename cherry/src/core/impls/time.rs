use crate::core::value::{Type, Value};
use crate::{Error, Result};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;


impl Type for OffsetDateTime {
    fn to_db(&self) -> Result<Value> {
        // Formatted as RFC3339 string
        let formatted = self.format(&Rfc3339).unwrap_or_else(|_| self.to_string());
        Ok(Value::Text(formatted))
    }

    fn from_db(value: Value) -> Result<Self> {
        match value {
            Value::Text(s) => {
                OffsetDateTime::parse(&s, &Rfc3339)
                    .map_err(|e| Error::ParseError(e.to_string()))
            }
            _ => Err(Error::TypeMismatch("Expected Text for time::OffsetDateTime".into())),
        }
    }
}