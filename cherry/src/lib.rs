// #![allow(unused_imports, deprecated, unused_must_use, unused_mut, unused_variables, dead_code, unreachable_code)]

// For `mod tests` run proc macro `::cherry::...`, It means self.
extern crate self as cherry;

pub(crate) mod error;
pub(crate) mod types;
pub(crate) mod ops;
pub(crate) mod executor;

pub use {
    error::Error,

    cherry_derive::Model,
    // types::model::Model,
    // types::model::ColumnMeta,
    types::model,
    types::value::{DbType, DbValue},
    types::row::Row,

    executor::database::Database,
    executor::transaction::Transaction,
};

// pub mod model {
//
// }

pub mod query {
    // CRUD
    pub use crate::ops::{insert, select, update, delete};

    // DSL
    pub mod dsl {
        pub use crate::ops::dsl::column::Column;
        pub use crate::ops::dsl::condition::Condition;
        pub use crate::ops::dsl::assignment::Assignment;
    }
}


pub type Result<T, E = crate::error::Error> = core::result::Result<T, E>;