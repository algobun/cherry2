use crate::Result;
use rusqlite::types::FromSql;
use rusqlite::RowIndex;

pub struct Row<'a> {
    pub(crate) inner: &'a rusqlite::Row<'a>,
}

impl<'a> Row<'a> {
    pub(crate) fn new(inner: &'a rusqlite::Row<'a>) -> Self {
        Self { inner }
    }

    pub fn get<I: RowIndex, T: FromSql>(&self, idx: I) -> Result<T> {
        Ok(self.inner.get(idx)?)
    }
}
