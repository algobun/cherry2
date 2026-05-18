use crate::core::row::Row;
use crate::core::value::Value;
use crate::Result;
use std::borrow::Cow;

pub trait FromRow: Sized {
    fn columns() -> Cow<'static, [ColumnMeta]>;
    fn from_row(row: &Row) -> Result<Self>;
}

pub trait Model: FromRow {
    fn table_name() -> &'static str;
    fn to_values(&self) -> Result<Vec<(&'static str, Value)>>;
}

#[derive(Debug, Copy, Clone)]
pub struct ColumnMeta {
    pub table_name: &'static str,
    pub column_name: &'static str,
    pub is_autoincrement: bool,
    pub is_json: bool,
    pub is_ignore: bool,
    // pub(crate) is_primary_key: bool,
    // pub(crate) is_nullable: bool,
}




#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use crate::types::Value;
    use crate::Model;
    use crate::types::Type;
    

    #[derive(Model)]
    struct User {
        id: i64,
        #[orm(autoincrement, column = "full_name")]
        name: String,
        // #[orm(json)]
        // book: Book,
    }

    #[test]
    fn test() {
        let meta = User::name.meta;
        println!("{:?}", meta);
    }



    #[derive(Serialize, Deserialize)]
    struct Book {
        id: usize,
        title: String,
    }


}