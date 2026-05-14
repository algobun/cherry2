use crate::{DbType, DbValue, Error, Result};
use url::Url;

impl DbType for Url {
    fn to_db_value(&self) -> DbValue {
        // Url's Display trait already implements standard serialization.
        DbValue::Text(self.to_string())
    }

    fn from_db_value(value: DbValue) -> Result<Self> {
        match value {
            DbValue::Text(s) => {
                Url::parse(&s).map_err(|e| Error::ParseError(e.to_string()))
            }
            _ => Err(Error::TypeMismatch("Expected Text for url::Url".into())),
        }
    }
}