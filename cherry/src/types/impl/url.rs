use crate::{Error, Result, Type, Value};
use url::Url;

impl Type for Url {
    fn to_db(&self) -> Result<Value> {
        // Url's Display trait already implements standard serialization.
        Ok(Value::Text(self.to_string()))
    }

    fn from_db(value: Value) -> Result<Self> {
        match value {
            Value::Text(s) => {
                Url::parse(&s).map_err(|e| Error::ParseError(e.to_string()))
            }
            _ => Err(Error::TypeMismatch("Expected Text for url::Url".into())),
        }
    }
}