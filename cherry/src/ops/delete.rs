use crate::core::model::{FromRow, Model};
use crate::core::value::Value;
use crate::executor::Executor;
use crate::ops::dsl::condition::Condition;
use crate::ops::dsl::returning::{NoReturning, Returning, ReturningColumns};
use crate::ops::dsl::QueryContext;
use crate::Result;
use rusqlite::params_from_iter;
use std::marker::PhantomData;

/// Represents a DELETE query builder with type-state.
/// `S` tracks the returning state (NoReturning or Returning).
pub struct Delete<'a, T, S = NoReturning> {
    executor: Executor<'a>,
    condition: Option<Condition>,
    /// Core: generic field holding the necessary data for the current returning state.
    returning: S,
    _marker: PhantomData<T>,
}

impl<'a, T: Model> Delete<'a, T, NoReturning> {
    pub(crate) fn create(executor: Executor<'a>) -> Self {
        Self {
            executor,
            condition: None,
            returning: NoReturning,
            _marker: PhantomData,
        }
    }

    /// Adds a `WHERE` condition to the update statement.
    /// Calling this multiple times will automatically combine them with `AND`.
    pub fn filter(mut self, cond: Condition) -> Self {
        self.condition = match self.condition {
            Some(existing) => Some(existing & cond),
            None => Some(cond),
        };
        self
    }

    /// State transition: Specifies single or multiple columns to return.
    pub fn returning<C: ReturningColumns>(self, cols: C) -> Delete<'a, T, Returning<C::Output>> {
        Delete {
            executor: self.executor,
            condition: self.condition,
            returning: Returning::from_columns(cols),
            _marker: PhantomData,
        }
    }

    /// State transition: Specifies a DTO/Model to map the returned row into.
    pub fn returning_as<U: FromRow>(self) -> Delete<'a, T, Returning<U>> {
        Delete {
            executor: self.executor,
            condition: self.condition,
            returning: Returning::from_row(),
            _marker: PhantomData,
        }
    }

    /// Executes the query and returns the number of affected rows.
    pub fn execute(self) -> Result<usize> {
        let (sql, params) = build_sql_and_params::<T>(&self.condition, None)?;
        self.executor.with_conn(|conn| Ok(conn.execute(&sql, params_from_iter(params))?))
    }
}

impl<'a, T: Model, O> Delete<'a, T, Returning<O>> {
    /// Executes the query and parses the returned dataset.
    pub fn execute(self) -> Result<Vec<O>> {
        let (sql, params) = build_sql_and_params::<T>(&self.condition, Some(&self.returning.projection))?;

        self.executor.with_conn(|conn| {
            let mut stmt = conn.prepare(&sql)?;
            let mut rows = stmt.query(params_from_iter(params))?;
            let mut result = Vec::new();
            while let Some(row) = rows.next()? {
                result.push((self.returning.parser)(row)?);
            }
            Ok(result)
        })
    }
}

/// Helper function to build the SQL string and collect parameters.
fn build_sql_and_params<T: Model>(
    condition: &Option<Condition>,
    returning_projection: Option<&str>
) -> Result<(String, Vec<Value>)> {
    let mut sql = String::from("delete from ");
    sql.push_str(T::table_name());

    let mut params = Vec::new();

    if let Some(cond) = condition {
        sql.push_str(" where ");
        cond.clone().build(&mut sql, &mut params, QueryContext::Delete)?;
    }

    if let Some(projection) = returning_projection {
        sql.push_str(" returning ");
        sql.push_str(projection);
    }

    Ok((sql, params))
}