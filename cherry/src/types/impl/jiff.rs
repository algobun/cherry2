use crate::{Error, Result, Type, Value};
use jiff::Timestamp;

impl Type for Timestamp {
    fn to_db(&self) -> Result<Value> {
        // jiff converts to a standard ISO 8601 string by default
        Ok(Value::Text(self.to_string()))
    }

    fn from_db(value: Value) -> Result<Self> {
        match value {
            Value::Text(s) => {
                s.parse::<Timestamp>().map_err(|e| Error::ParseError(e.to_string()))
            }
            _ => Err(Error::TypeMismatch("Expected Text for jiff::Timestamp".into())),
        }
    }
}