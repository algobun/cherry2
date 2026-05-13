use crate::types::row::Row;
use crate::types::value::DbValue;
use crate::Result;

pub trait Model: Sized {

    fn table_name() -> &'static str;

    fn columns() -> &'static [ColumnMeta];

    fn to_values(&self) -> Vec<(&'static str, DbValue)>;

    /// Convert Row to the current model entity
    fn from_row(row: &Row) -> Result<Self>;
}

#[derive(Copy, Clone)]
pub struct ColumnMeta {
    pub(crate) name: &'static str,
    pub(crate) is_ignore: bool,
    pub(crate) is_autoincrement: bool,
    // pub(crate) is_primary_key: bool,
    // pub(crate) is_nullable: bool,
}

#[cfg(test)]
mod tests {
    use crate::Model;

    #[derive(Model)]
    struct User {
        id: i64,
        name: String,
    }

}