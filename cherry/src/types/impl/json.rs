use crate::{DbType, DbValue, Error, Result};

impl DbType for serde_json::Value {
    fn to_db_value(&self) -> DbValue {
        // 将 JSON 对象序列化为字符串存入 SQLite
        DbValue::Text(self.to_string())
    }

    fn from_db_value(value: DbValue) -> Result<Self> {
        match value {
            DbValue::Text(s) => {
                serde_json::from_str(&s).map_err(|e| Error::ParseError(e.to_string()))
            }
            _ => Err(Error::TypeMismatch("Expected Text for JSON Value".into())),
        }
    }
}