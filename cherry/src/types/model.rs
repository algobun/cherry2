use crate::types::row::FromRow;
use crate::types::value::DbValue;

pub trait Model: FromRow {
    fn table_name() -> &'static str;

    fn columns() -> &'static [ColumnMeta];

    fn to_values(&self) -> Vec<(&'static str, DbValue)>;
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
    // use crate::Model;

    // use crate::model::Model;
    // use cherry::model::{ColumnMeta, Row};
    // use cherry::types::row::FromRow;
    // use cherry::DbValue;
    use cherry_derive::Model;

    // #[derive(Model)]
    struct User {
        id: i64,
        name: String,
    }

    #[derive(Model)]
    struct Book {
        id: usize,
        title: String,
    }

    // impl FromRow for User {
    //
    //     fn from_row(row: &Row) -> cherry::Result<Self> {
    //         todo!()
    //     }
    // }
    //
    // impl Model for User {
    //     fn table_name() -> &'static str {
    //         todo!()
    //     }
    //
    //     fn columns() -> &'static [ColumnMeta] {
    //         todo!()
    //     }
    //
    //     fn to_values(&self) -> Vec<(&'static str, DbValue)> {
    //         todo!()
    //     }
    // }



}