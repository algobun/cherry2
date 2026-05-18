use crate::executor::Executor;
use crate::ops::delete::Delete;
use crate::ops::insert::{AsRows, Insert};
use crate::ops::raw::Raw;
use crate::ops::select::Select;
use crate::ops::update::Update;
use crate::core::model::Model;

/// Represents a database transaction context.
/// Provides the exact same CRUD builder APIs as `Database`, ensuring consistent usage.
pub struct Transaction<'a> {
    executor: Executor<'a>
}

impl<'a> Transaction<'a> {

    pub(crate) fn create(executor: Executor<'a>) -> Transaction<'a> {
        Transaction { executor }
    }

    /// Initiates an INSERT operation builder within the current transaction.
    pub fn insert<T: Model, M: AsRows<'a, T>>(&'a self, m: M) -> Insert<'a, T> {
        Insert::create(self.executor, m)
    }

    /// Initiates a SELECT operation builder within the current transaction.
    pub fn select<T: Model>(&'_ self) -> Select<'_, T> {
        Select::create(self.executor)
    }

    /// Initiates an UPDATE operation builder within the current transaction.
    pub fn update<T: Model>(&'a self) -> Update<'a, T> {
        Update::create(self.executor)
    }

    /// Initiates a DELETE operation builder within the current transaction.
    pub fn delete<T: Model>(&'a self) -> Delete<'a, T> {
        Delete::create(self.executor)
    }

    /// Initiates a RAW SQL operation builder within the current transaction.
    pub fn raw<S: Into<String>>(&'_ self, sql: S) -> Raw<'_> {
        Raw::create(self.executor, sql)
    }
}