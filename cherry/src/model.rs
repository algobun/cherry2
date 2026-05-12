use crate::Result;


pub trait Model: Sized {

    fn table_name() -> &'static str;

    /// Get all column names mapped to the database (Use for concat SELECT a, b, c FROM ...)
    fn columns() -> &'static [&'static str];

    /// Convert rusqlite's Row to the current model entity
    fn from_row(row: &rusqlite::Row) -> Result<Self>;
}