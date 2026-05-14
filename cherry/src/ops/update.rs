use crate::executor::Executor;
use crate::model::FromRow;
use crate::model::Row;
use crate::ops::dsl::assignment::build_assignments;
use crate::ops::dsl::assignment::Assignment;
use crate::ops::dsl::column::Column;
use crate::ops::dsl::QueryContext;
use crate::query::dsl::Condition;
use crate::types::model::Model;
use crate::{DbType, DbValue, Error, Result};
use rusqlite::params_from_iter;
use std::marker::PhantomData;

/// Represents the builder for an UPDATE query.
pub struct Update<'a, T> {
    executor: Executor<'a>,
    assignments: Vec<Assignment>,
    condition: Option<Condition>,
    _marker: PhantomData<T>,
}

impl<'a, T: Model> Update<'a, T> {

    pub(crate) fn create(executor: Executor<'a>) -> Self {
        Self {
            executor,
            assignments: Vec::new(),
            condition: None,
            _marker: PhantomData,
        }
    }

    /// Adds a `SET col = val` assignment to the update statement.
    /// Can be chained multiple times.
    pub fn set(mut self, assignment: Assignment) -> Self {
        self.assignments.push(assignment);
        self
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

    /// Compiles the AST into executable SQL and a parameter vector.
    fn build_sql_and_params(&self, returning_suffix: Option<&str>) -> Result<(String, Vec<DbValue>)> {
        // [Safety Check] Prevent generating invalid SQL like "update table set where ..."
        if self.assignments.is_empty() {
            return Err(Error::QueryError("Cannot execute update without any assignments (SET)".into()));
        }

        let mut sql = String::from("update ");
        sql.push_str(T::table_name());
        sql.push_str(" set ");

        let mut params = Vec::new();
        build_assignments(&self.assignments, &mut sql, &mut params);

        if let Some(cond) = &self.condition {
            sql.push_str(" where ");
            // Pass the Update context to the AST builder
            cond.clone().build(&mut sql, &mut params, QueryContext::Update);
        }

        if let Some(suffix) = returning_suffix {
            sql.push_str(" returning ");
            sql.push_str(suffix);
        }

        Ok((sql, params))
    }

    /// Executes the update statement, returning the total number of affected rows.
    pub fn execute(self) -> Result<usize> {
        let (sql, params) = self.build_sql_and_params(None)?;
        let conn = self.executor.conn();
        
        let affected = conn.execute(&sql, params_from_iter(params))?;
        Ok(affected)
    }

    /// Executes the update and returns a single strongly-typed column from the updated rows.
    pub fn returning<U: DbType>(self, col: Column<U>) -> Result<Vec<U>> {
        let (sql, params) = self.build_sql_and_params(Some(col.name))?;

        let conn = self.executor.conn();
        let mut stmt = conn.prepare(&sql)?;
        let mut rows = stmt.query(params_from_iter(params))?;

        let mut result = Vec::new();
        while let Some(row) = rows.next()? {
            let db_val: DbValue = row.get(0)
                .map_err(|e| Error::QueryError(e.to_string()))?;
            let val = U::from_db_value(db_val)?;
            result.push(val);
        }

        Ok(result)
    }

    /// Executes the update and projects the returned rows into a specified DTO.
    /// The target type `U` must implement `FromRow`.
    pub fn returning_as<U: FromRow>(self) -> Result<Vec<U>> {
        let cols = U::columns().iter().map(|c| c.name).collect::<Vec<_>>().join(", ");
        let (sql, params) = self.build_sql_and_params(Some(&cols))?;

        let conn = self.executor.conn();
        let mut stmt = conn.prepare(&sql)?;
        let mut rows = stmt.query(params_from_iter(params))?;

        let mut result = Vec::new();
        while let Some(row) = rows.next()? {
            result.push(U::from_row(&Row::new(row))?);
        }

        Ok(result)
    }
}