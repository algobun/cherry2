use crate::Result;
use rusqlite::types::FromSql;
use rusqlite::RowIndex;

pub struct Row<'a> {
    inner: rusqlite::Row<'a>,
}

impl Row<'_> {
    
    pub fn get<I: RowIndex, T: FromSql>(&self, idx: I) -> Result<T> {
        Ok(self.inner.get(idx)?)
    }
    
}