use crate::Result;
use rusqlite::types::FromSql;
use rusqlite::RowIndex;

pub struct Row<'a> {
    inner: &'a rusqlite::Row<'a>,
}

impl<'a> Row<'a> {
    pub(crate) fn new(inner: &'a rusqlite::Row<'a>) -> Self {
        Self { inner }
    }

    pub fn get<I: RowIndex, T: FromSql>(&self, idx: I) -> Result<T> {
        Ok(self.inner.get(idx)?)
    }
}

pub trait FromRow: Sized {
    // Convert Row to the current model entity
    fn from_row(row: &Row) -> Result<Self>;
}