// #![allow(unused_imports, deprecated, unused_must_use, unused_mut, unused_variables, dead_code, unreachable_code)]

// For `mod tests` run proc macro `::cherry::...`, It means self.
extern crate self as cherry;

pub(crate) mod error;
pub(crate) mod types;
pub(crate) mod ops;
pub(crate) mod executor;

pub use {
    cherry_derive::Model,
    error::Error,
    executor::database::Database,
    executor::transaction::Transaction,
    types::value::{DbType, DbValue},
};

pub mod model {
    pub use crate::types::model::{ColumnMeta, Model};
    pub use crate::types::row::{FromRow, Row};
}

pub mod query {
    // CRUD
    pub use crate::ops::{delete, insert, select, update};

    // DSL
    pub mod dsl {
        pub use crate::ops::dsl::assignment::Assignment;
        pub use crate::ops::dsl::column::Column;
        pub use crate::ops::dsl::condition::Condition;
        pub use crate::ops::dsl::order::{OrderBy, OrderDir};
    }
}


pub type Result<T, E = crate::error::Error> = core::result::Result<T, E>;