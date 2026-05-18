use crate::{Error, Result, Type, Value};

impl Type for serde_json::Value {
    fn to_db(&self) -> Result<Value> {
        // Serialize JSON objects as strings in SQLite
        Ok(Value::Text(self.to_string()))
    }

    fn from_db(value: Value) -> Result<Self> {
        match value {
            Value::Text(s) => {
                serde_json::from_str(&s).map_err(|e| Error::ParseError(e.to_string()))
            }
            _ => Err(Error::TypeMismatch("Expected Text for JSON Value".into())),
        }
    }
}