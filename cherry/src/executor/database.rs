use crate::executor::transaction::Transaction;
use crate::executor::Executor;
use crate::ops::delete::Delete;
use crate::ops::insert::{AsRows, Insert};
use crate::ops::raw::Raw;
use crate::ops::select::Select;
use crate::ops::update::Update;
use crate::types::model::Model;
use crate::Result;
use rusqlite::Connection;
use std::path::Path;

/// The primary entry point for database operations.
/// Manages the underlying SQLite connection and spawns operation builders.
pub struct Database {
    conn: Connection,
}

impl Database {

    /// Opens a connection to a new or existing SQLite database at the given path.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(Self {
            conn: Connection::open(path)?,
        })
    }

    /// Initiates an INSERT operation builder.
    /// Accepts a single model, a slice, or a vector of models.
    pub fn insert<'a, T: Model, M: AsRows<'a, T>>(&'a self, m: M) -> Insert<'a, T> {
        Insert::create(Executor::Connection(&self.conn), m)
    }

    /// Initiates a SELECT operation builder.
    pub fn select<T: Model>(&'_ self) -> Select<'_, T> {
        Select::create(Executor::Connection(&self.conn))
    }

    /// Initiates an UPDATE operation builder.
    pub fn update<T: Model>(&'_ self) -> Update<'_, T> {
        Update::create(Executor::Connection(&self.conn))
    }

    /// Initiates a DELETE operation builder.
    pub fn delete<T: Model>(&'_ self) -> Delete<'_, T> {
        Delete::create(Executor::Connection(&self.conn))
    }

    /// Initiates a RAW SQL operation builder.
    /// Provides an escape hatch for complex queries, supporting manual parameter binding.
    pub fn raw<S: Into<String>>(&'_ self, sql: S) -> Raw<'_> {
        Raw::create(Executor::Connection(&self.conn), sql)
    }

    /// Executes a closure within a database transaction.
    /// If the closure returns `Ok`, the transaction is committed.
    /// If the closure returns `Err`, the transaction is automatically rolled back.
    pub fn transaction<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&Transaction) -> Result<()>,
    {
        let inner_tx = self.conn.transaction()?;
        let tx = Transaction::create(Executor::Transaction(&inner_tx));

        // Execute the user-provided closure
        f(&tx)?;

        // If no error occurred, commit the transaction
        Ok(inner_tx.commit()?)
    }
}