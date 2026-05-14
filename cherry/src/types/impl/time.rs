use crate::{DbType, DbValue, Error, Result};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;


impl DbType for OffsetDateTime {
    fn to_db_value(&self) -> DbValue {
        // Formatted as RFC3339 string
        let formatted = self.format(&Rfc3339).unwrap_or_else(|_| self.to_string());
        DbValue::Text(formatted)
    }

    fn from_db_value(value: DbValue) -> Result<Self> {
        match value {
            DbValue::Text(s) => {
                OffsetDateTime::parse(&s, &Rfc3339)
                    .map_err(|e| Error::ParseError(e.to_string()))
            }
            _ => Err(Error::TypeMismatch("Expected Text for time::OffsetDateTime".into())),
        }
    }
}