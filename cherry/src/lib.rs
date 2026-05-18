// #![allow(unused_imports, deprecated, unused_must_use, unused_mut, unused_variables, dead_code, unreachable_code)]

// For `mod tests` run proc macro `::cherry::...`, It means self.
#![allow(private_bounds)]
extern crate self as cherry;

pub(crate) mod error;
pub(crate) mod core;
pub(crate) mod database;
pub(crate) mod executor;
pub(crate) mod ops;

pub use cherry_derive::{FromRow, Model};
pub use error::Error;
pub type Result<T, E = crate::error::Error> = std::result::Result<T, E>;

pub mod db {
    pub use crate::database::builder::DatabaseBuilder;
    pub use crate::database::db::Database;
    pub use crate::executor::transaction::Transaction;
}

pub mod types {
    pub use crate::core::model::{ColumnMeta, FromRow, Model};
    pub use crate::core::row::Row;
    pub use crate::core::value::{Type, Value};

    #[doc(hidden)]
    #[cfg(feature = "json")]
    pub mod json {
        pub use crate::core::impls::json::{from_db, to_db, Json};
    }
}

pub mod query {
    // CRUD
    pub use crate::ops::delete::Delete;
    pub use crate::ops::insert::{Insert, InsertUpdate};
    pub use crate::ops::raw::Raw;
    pub use crate::ops::select::Select;
    pub use crate::ops::update::Update;

    // DSL
    pub mod dsl {
        pub use crate::ops::dsl::assignment::Assignment;
        pub use crate::ops::dsl::column::Column;
        pub use crate::ops::dsl::condition::Condition;
        pub use crate::ops::dsl::order::{OrderBy, OrderDir};
    }
}
