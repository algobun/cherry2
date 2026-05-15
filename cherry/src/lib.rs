// #![allow(unused_imports, deprecated, unused_must_use, unused_mut, unused_variables, dead_code, unreachable_code)]
#![allow(private_bounds)]

// For `mod tests` run proc macro `::cherry::...`, It means self.
extern crate self as cherry;

pub(crate) mod error;
pub(crate) mod types;
pub(crate) mod ops;
pub(crate) mod executor;
mod database;

pub use {
    cherry_derive::Model,
    database::builder::DatabaseBuilder,
    database::db::Database,
    error::Error,
    executor::transaction::Transaction,
    types::value::{DbType, DbValue},
};

pub mod model {
    pub use crate::types::model::{ColumnMeta, Model};
    pub use crate::types::row::{FromRow, Row};
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


pub type Result<T, E = crate::error::Error> = core::result::Result<T, E>;