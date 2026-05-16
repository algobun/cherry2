use crate::types::row::Row;
use crate::types::value::DbValue;
use std::borrow::Cow;

pub trait FromRow: Sized {
    fn columns() -> Cow<'static, [ColumnMeta]>;
    fn from_row(row: &Row) -> crate::Result<Self>;
}

pub trait Model: FromRow {
    fn table_name() -> &'static str;
    fn to_values(&self) -> Vec<(&'static str, DbValue)>;
}

#[derive(Debug, Copy, Clone)]
pub struct ColumnMeta {
    pub(crate) table_name: &'static str,
    pub(crate) column_name: &'static str,
    pub(crate) is_ignore: bool,
    pub(crate) is_autoincrement: bool,
    // pub(crate) is_primary_key: bool,
    // pub(crate) is_nullable: bool,
}




#[cfg(test)]
mod tests {
    use crate::Model;
    // use crate::model::FromRow;
    // use crate::model::Model;
    // use cherry::model::{ColumnMeta, Row};
    // use cherry::types::row::FromRow;
    // use cherry::DbValue;
    // use cherry_derive::Model;

    #[derive(Model)]
    struct User {
        id: i64,
        name: String,
    }

    fn a() {

    }


    // #[derive(Model)]
    struct Book {
        id: usize,
        title: String,
    }

    // impl FromRow for User {
    //     fn columns() -> &'static [ColumnMeta] {
    //         todo!()
    //     }
    // 
    //     fn from_row(row: &Row) -> cherry::Result<Self> {
    //         todo!()
    //     }
    // }

    // impl Model for User {
    //     fn table_name() -> &'static str {
    //         todo!()
    //     }
    // 
    //     // fn columns() -> &'static [ColumnMeta] {
    //     //     todo!()
    //     // }
    // 
    //     fn to_values(&self) -> Vec<(&'static str, DbValue)> {
    //         todo!()
    //     }
    // }
    
    // fn a() {
    //     let a = User::columns();
    // }



}