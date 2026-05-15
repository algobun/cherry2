use crate::executor::Executor;
use crate::model::{FromRow, Row};
use crate::{DbValue, Result};
use rusqlite::params_from_iter;

/// Represents a builder for executing raw SQL queries.
/// Provides an escape hatch for complex queries that the DSL cannot express.
pub struct Raw<'a> {
    executor: Executor<'a>,
    sql: String,
    params: Vec<DbValue>,
}

impl<'a> Raw<'a> {
    pub(crate) fn create(executor: Executor<'a>, sql: impl Into<String>) -> Self {
        Self {
            executor,
            sql: sql.into(),
            params: Vec::new(),
        }
    }

    /// Binds a single parameter to the SQL query.
    /// Can be chained multiple times.
    ///
    /// # Example
    /// ```rust
    /// db.raw("SELECT * FROM users WHERE age > ? AND status = ?")
    ///   .bind(18)
    ///   .bind("active");
    /// ```
    pub fn bind<V: Into<DbValue>>(mut self, val: V) -> Self {
        self.params.push(val.into());
        self
    }

    /// Binds multiple parameters from an iterator.
    /// Highly useful for queries with dynamic `IN (?, ?, ?)` clauses or bulk operations.
    ///
    /// # Example
    /// ```rust
    /// let ids = vec![1, 2, 3];
    /// db.raw("SELECT * FROM users WHERE id IN (?, ?, ?)")
    ///   .bind_all(ids);
    /// ```
    pub fn bind_all<I, V>(mut self, iter: I) -> Self
    where
        I: IntoIterator<Item = V>,
        V: Into<DbValue>,
    {
        self.params.extend(iter.into_iter().map(Into::into));
        self
    }

    /// Executes the raw SQL query, returning the number of affected rows.
    /// Typically used for `UPDATE`, `DELETE`, or `INSERT` statements.
    pub fn execute(self) -> Result<usize> {
        let conn = self.executor.conn();
        let affected = conn.execute(&self.sql, params_from_iter(self.params))?;
        Ok(affected)
    }

    /// Executes the raw SQL query and maps the returned dataset into a specified DTO.
    /// The target type `U` must implement `FromRow`.
    /// Typically used for `SELECT` or `RETURNING` statements.
    pub fn load_as<U: FromRow>(self) -> Result<Vec<U>> {
        let conn = self.executor.conn();
        let mut stmt = conn.prepare(&self.sql)?;
        let mut rows = stmt.query(params_from_iter(self.params))?;

        let mut result = Vec::new();
        while let Some(rusqlite_row) = rows.next()? {
            result.push(U::from_row(&Row::new(rusqlite_row))?);
        }

        Ok(result)
    }
}