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
    use std::borrow::Cow;
    use cherry::types::{ColumnMeta, Row};
    use crate::db::DatabaseBuilder;
    use crate::types::{FromRow, Value};
    use crate::{Error, Model};
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
        let db = DatabaseBuilder::new("").build().unwrap();
        let u = db.select::<User>().columns(("", User::name)).load().unwrap();
    }



    // #[derive(Serialize, Deserialize)]
    struct Book {
        id: usize,
        title: String,
    }

    impl FromRow for Book {
        fn columns() -> Cow<'static, [ColumnMeta]> {
            todo!()
        }

        fn from_row(row: &Row) -> cherry::Result<Self> {
            let val: Value = row.get("book.id").or_else(|_| row.get("id").map_err(|e| Error::ParseError(e.to_string())))?;
            let id: usize = Type::from_db(val)?;
            todo!()
        }
    }


}