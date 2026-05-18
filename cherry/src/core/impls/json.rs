use crate::core::value::{Type, Value};
use crate::{Error, Result};
use serde::de::DeserializeOwned;
use serde::Serialize;

pub struct Json<T>(pub T);

impl<T: Serialize> Type for Json<T> {
    fn to_db(&self) -> Result<Value> {
        to_db(&self.0)
    }

    fn from_db(_: Value) -> Result<Self> {
        unimplemented!("`Json` is only used for generation, never for parsing")
    }
}

impl<T> From<T> for Json<T> {
    fn from(val: T) -> Self {
        Json(val)
    }
}


pub fn to_db<T: Serialize>(val: &T) -> Result<Value> {
    let json_str = serde_json::to_string(val)
        .map_err(|e| Error::ConversionError(format!("JSON serialize failed: {}", e)))?;
    Ok(Value::Text(json_str))
}

pub fn from_db<T: DeserializeOwned>(val: Value) -> Result<T> {
    match val {
        Value::Text(s) => serde_json::from_str(&s)
            .map_err(|e| Error::ParseError(format!("JSON deserialize failed: {}", e))),
        Value::Blob(b) => serde_json::from_slice(&b) // Compatible with some strange writes
            .map_err(|e| Error::ParseError(format!("JSON deserialize failed: {}", e))),
        _ => Err(Error::TypeMismatch("Expected Text or Blob for JSON".into())),
    }
}
