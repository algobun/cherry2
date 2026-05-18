use crate::database::builder::DatabaseBuilder;
use crate::executor::transaction::Transaction;
use crate::executor::Executor;
use crate::ops::delete::Delete;
use crate::ops::insert::{AsRows, Insert};
use crate::ops::raw::Raw;
use crate::ops::select::Select;
use crate::ops::update::Update;
use crate::core::model::Model;
use crate::Result;
use std::path::Path;

pub(crate) enum Backend {
    /// Supports a single connection mode, available regardless of the `pool` feature.
    #[cfg(not(feature = "pool"))]
    Single(rusqlite::Connection),

    /// A connection pool variant, compiled only when the `pool` feature is enabled.
    #[cfg(feature = "pool")]
    Pooled(r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>),
}

/// The primary entry point for database operations.
/// Manages the underlying SQLite connection and spawns operation builders.
pub struct Database {
    pub(crate) backend: Backend,
}

impl Database {

    /// Opens a connection to a new or existing SQLite database at the given path.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        DatabaseBuilder::new(path).build()
    }

    /// Initiates an INSERT operation builder.
    /// Accepts a single model, a slice, or a vector of models.
    pub fn insert<'a, T: Model, M: AsRows<'a, T>>(&'a self, m: M) -> Insert<'a, T> {
        Insert::create(Executor::Backend(&self.backend), m)
    }

    /// Initiates a SELECT operation builder.
    pub fn select<T: Model>(&'_ self) -> Select<'_, T> {
        Select::create(Executor::Backend(&self.backend))
    }

    /// Initiates an UPDATE operation builder.
    pub fn update<T: Model>(&'_ self) -> Update<'_, T> {
        Update::create(Executor::Backend(&self.backend))
    }

    /// Initiates a DELETE operation builder.
    pub fn delete<T: Model>(&'_ self) -> Delete<'_, T> {
        Delete::create(Executor::Backend(&self.backend))
    }

    /// Initiates a RAW SQL operation builder.
    /// Provides an escape hatch for complex queries, supporting manual parameter binding.
    pub fn raw<S: Into<String>>(&'_ self, sql: S) -> Raw<'_> {
        Raw::create(Executor::Backend(&self.backend), sql)
    }

    /// Executes a closure within a database transaction.
    /// If the closure returns `Ok`, the transaction is committed.
    /// If the closure returns `Err`, the transaction is automatically rolled back.
    /// A single-connection transaction must be `&mut self` (mutually exclusive borrowing).
    #[cfg(not(feature = "pool"))]
    pub fn transaction<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&Transaction) -> Result<()>,
    {
        let Backend::Single(conn) = &mut self.backend;
        let inner_tx = conn.transaction()?;
        let tx = Transaction::create(Executor::Transaction(&inner_tx));
        // Execute the user-provided closure
        f(&tx)?;
        // If no error occurred, commit the transaction
        Ok(inner_tx.commit()?)
    }

    /// The transaction of a connection pool can be `&self`
    /// (shared borrowing, allowing multi-threaded concurrency).
    #[cfg(feature = "pool")]
    pub fn transaction<F>(&self, f: F) -> Result<()>
    where
        F: FnOnce(&Transaction) -> Result<()>,
    {
        let Backend::Pooled(pool) = &self.backend;

        // Takes a connection out of the pool, at which point the mut conn is exclusive
        // to that thread and does not affect &self.inner.
        let mut conn = pool.get().map_err(|e| Error::ConnectionError(e.to_string()))?;

        let inner_tx = conn.transaction()?;
        let tx = Transaction::create(Executor::Transaction(&inner_tx));
        f(&tx)?;
        Ok(inner_tx.commit()?)
    }

}

#[cfg(test)]
mod tests {
    use cherry_derive::Model;
    use super::*;

    // #[derive(Model)]
    struct User {
        id: usize,
        name: String,
    }

    #[test]
    fn test() {
        // let a = DatabaseBuilder::new("").build().unwrap();
        // let r = a.select::<User>().filter(User::id.eq(1_usize) & User::name.eq("zh")).load().unwrap();
    }
}