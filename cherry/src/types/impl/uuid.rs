use crate::{DbType, DbValue, Error, Result};
use uuid::Uuid;

impl DbType for Uuid {
    fn to_db_value(&self) -> DbValue {
        // UUID 转为 16 字节数组存入 Blob
        DbValue::Blob(self.as_bytes().to_vec())
    }

    fn from_db_value(value: DbValue) -> Result<Self> {
        match value {
            DbValue::Blob(b) => {
                Uuid::from_slice(&b).map_err(|e| Error::ParseError(e.to_string()))
            }
            // 兼容有些用户手改数据库存成了 Text 的情况
            DbValue::Text(s) => {
                Uuid::parse_str(&s).map_err(|e| Error::ParseError(e.to_string()))
            }
            _ => Err(Error::TypeMismatch("Expected Blob or Text for Uuid".into())),
        }
    }
}