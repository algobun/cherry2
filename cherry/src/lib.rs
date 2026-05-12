// #![allow(unused_imports, deprecated, unused_must_use, unused_mut, unused_variables, dead_code, unreachable_code)]

pub(crate) mod error;
pub(crate) mod types;
pub(crate) mod query;
pub mod model;
pub mod executor;

pub use {

};

pub type Result<T, E = crate::error::Error> = core::result::Result<T, E>;