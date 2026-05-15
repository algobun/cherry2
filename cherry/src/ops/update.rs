use crate::executor::Executor;
use crate::model::FromRow;
use crate::ops::dsl::assignment::build_assignments;
use crate::ops::dsl::assignment::Assignment;
use crate::ops::dsl::returning::{NoReturning, Returning, ReturningColumns};
use crate::ops::dsl::QueryContext;
use crate::query::dsl::Condition;
use crate::types::model::Model;
use crate::{DbValue, Error, Result};
use rusqlite::params_from_iter;
use std::marker::PhantomData;

/// Represents the builder for an UPDATE query.
/// `S` tracks the returning state (NoReturning or Returning).
pub struct Update<'a, T, S = NoReturning> {
    executor: Executor<'a>,
    assignments: Vec<Assignment>,
    condition: Option<Condition>,
    returning: S,
    _marker: PhantomData<T>,
}

impl<'a, T: Model> Update<'a, T, NoReturning> {
    pub(crate) fn create(executor: Executor<'a>) -> Self {
        Self {
            executor,
            assignments: Vec::new(),
            condition: None,
            returning: NoReturning,
            _marker: PhantomData,
        }
    }

    /// Adds a `set col = val` assignment to the update statement.
    /// Can be chained multiple times.
    pub fn set(mut self, assignment: Assignment) -> Self {
        self.assignments.push(assignment);
        self
    }

    /// Adds a `where` condition to the update statement.
    /// Calling this multiple times will automatically combine them with `AND`.
    pub fn filter(mut self, cond: Condition) -> Self {
        self.condition = match self.condition {
            Some(existing) => Some(existing & cond),
            None => Some(cond),
        };
        self
    }

    /// State transition: Specifies single or multiple columns to return.
    pub fn returning<C: ReturningColumns>(self, cols: C) -> Update<'a, T, Returning<C::Output>> {
        Update {
            executor: self.executor,
            assignments: self.assignments,
            condition: self.condition,
            returning: Returning::from_columns(cols),
            _marker: PhantomData,
        }
    }

    /// State transition: Specifies a DTO/Model to map the returned row into.
    pub fn returning_as<U: FromRow>(self) -> Update<'a, T, Returning<U>> {
        Update {
            executor: self.executor,
            assignments: self.assignments,
            condition: self.condition,
            returning: Returning::from_row(),
            _marker: PhantomData,
        }
    }

    /// Executes the update statement, returning the total number of affected rows.
    pub fn execute(self) -> Result<usize> {
        let (sql, params) = build_sql_and_params::<T>(&self.assignments, &self.condition, None)?;
        let conn = self.executor.conn();

        let affected = conn.execute(&sql, params_from_iter(params))?;
        Ok(affected)
    }
}

impl<'a, T: Model, O> Update<'a, T, Returning<O>> {
    /// Executes the update and parses the returned dataset.
    pub fn execute(self) -> Result<Vec<O>> {
        let (sql, params) = build_sql_and_params::<T>(
            &self.assignments,
            &self.condition,
            Some(&self.returning.projection)
        )?;

        let conn = self.executor.conn();
        let mut stmt = conn.prepare(&sql)?;
        let mut rows = stmt.query(params_from_iter(params))?;

        let mut result = Vec::new();
        while let Some(row) = rows.next()? {
            result.push((self.returning.parser)(row)?);
        }

        Ok(result)
    }
}

/// Compiles the AST into executable SQL and a parameter vector.
fn build_sql_and_params<T: Model>(
    assignments: &[Assignment],
    condition: &Option<Condition>,
    returning_projection: Option<&str>,
) -> Result<(String, Vec<DbValue>)> {
    // [Safety Check] Prevent generating invalid SQL like "update table set where ..."
    if assignments.is_empty() {
        return Err(Error::QueryError("Cannot execute update without any assignments (SET)".into()));
    }

    let mut sql = String::from("update ");
    sql.push_str(T::table_name());
    sql.push_str(" set ");

    let mut params = Vec::new();
    build_assignments(assignments, &mut sql, &mut params);

    if let Some(cond) = condition {
        sql.push_str(" where ");
        cond.clone().build(&mut sql, &mut params, QueryContext::Update);
    }

    if let Some(projection) = returning_projection {
        sql.push_str(" returning ");
        sql.push_str(projection);
    }

    Ok((sql, params))
}