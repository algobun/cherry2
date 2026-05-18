use crate::{Type, Value, Error, Result};
use uuid::Uuid;

impl Type for Uuid {
    fn to_db(&self) -> Result<Value> {
        // Convert UUID to 16-byte array and save to blob
        Ok(Value::Blob(self.as_bytes().to_vec()))
    }

    fn from_db(value: Value) -> Result<Self> {
        match value {
            Value::Blob(b) => {
                Uuid::from_slice(&b).map_err(|e| Error::ParseError(e.to_string()))
            }
            // Compatible with the situation where some users manually change the database to Text
            Value::Text(s) => {
                Uuid::parse_str(&s).map_err(|e| Error::ParseError(e.to_string()))
            }
            _ => Err(Error::TypeMismatch("Expected Blob or Text for Uuid".into())),
        }
    }
}