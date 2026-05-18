use crate::core::model::Model;
use crate::core::model::FromRow;
use crate::core::value::Value;
use crate::executor::Executor;
use crate::ops::dsl::assignment::{build_assignments, Assignment};
use crate::ops::dsl::column::Column;
use crate::ops::dsl::condition::Condition;
use crate::ops::dsl::returning::{NoReturning, Returning, ReturningColumns};
use crate::ops::dsl::QueryContext;
use crate::Result;
use rusqlite::params_from_iter;

/// Represents the builder for an INSERT query.
/// `S` tracks the returning state (NoReturning or Returning).
pub struct Insert<'a, T, S = NoReturning> {
    pub(crate) executor: Executor<'a>,
    pub(crate) rows: &'a [T],
    pub(crate) pattern: Pattern,
    pub(crate) returning: S,
}

impl<'a, T: Model> Insert<'a, T, NoReturning> {
    pub(crate) fn create<M>(executor: Executor<'a>, m: M) -> Self
    where
        M: AsRows<'a, T>,
    {
        Insert {
            executor,
            rows: m.as_rows(),
            pattern: Pattern::Normal,
            returning: NoReturning,
        }
    }

    /// State transition: Specifies single or multiple columns to return.
    pub fn returning<C: ReturningColumns>(self, cols: C) -> Insert<'a, T, Returning<C::Output>> {
        Insert {
            executor: self.executor,
            rows: self.rows,
            pattern: self.pattern,
            returning: Returning::from_columns(cols),
        }
    }

    /// State transition: Specifies a DTO/Model to map the returned row into.
    pub fn returning_as<U: FromRow>(self) -> Insert<'a, T, Returning<U>> {
        Insert {
            executor: self.executor,
            rows: self.rows,
            pattern: self.pattern,
            returning: Returning::from_row(),
        }
    }

    /// Executes the insert statement, returning the total number of affected rows.
    /// Automatically chunks the execution to avoid SQLite's parameter limit.
    pub fn execute(self) -> Result<usize> {
        let chunks = self.generate_chunks()?;
        execute_affected(self.executor, chunks)
    }
}

// General methods (not affected by the Returning state)
impl<'a, T: Model, S> Insert<'a, T, S> {
    /// Converts the statement to `insert or ignore into`
    pub fn or_ignore(mut self) -> Self {
        self.pattern = Pattern::OrIgnore;
        self
    }

    /// Converts the statement to `insert or replace into`
    pub fn or_replace(mut self) -> Self {
        self.pattern = Pattern::OrReplace;
        self
    }

    /// Appends the `on conflict` clause, returning an InsertUpdate builder.
    /// Passes down the current returning state.
    pub fn on_conflict<Target: ConflictTarget>(self, target: Target) -> InsertUpdate<'a, T, S> {
        InsertUpdate {
            builder: self,
            conflict_cols: target.to_columns(),
            assignments: vec![],
            condition: None,
        }
    }

    /// Generates executable chunks of SQL and parameters to bypass SQLite limits.
    fn generate_chunks(&self) -> Result<Vec<Chunk>> {
        let mut chunks = Vec::new();

        if self.rows.is_empty() {
            chunks.push(Chunk {
                base_sql: build_base_insert(self.pattern, T::table_name(), &[], 0),
                params: vec![],
            });
            return Ok(chunks);
        }

        let first_vals = self.rows[0].to_values()?;
        let col_names: Vec<&str> = first_vals.iter().map(|(k, _)| *k).collect();
        let cols_count = col_names.len();

        if cols_count == 0 {
            let sql = build_base_insert(self.pattern, T::table_name(), &[], 0);
            for _ in self.rows {
                chunks.push(Chunk { base_sql: sql.clone(), params: vec![] });
            }
            return Ok(chunks);
        }

        // SQLite limit is 32766. We leave a buffer of 200 for Upsert/WHERE parameters.
        let max_params = 32566;
        let chunk_size = std::cmp::max(1, max_params / cols_count);

        for chunk in self.rows.chunks(chunk_size) {
            let base_sql = build_base_insert(self.pattern, T::table_name(), &col_names, chunk.len());
            let mut params = Vec::with_capacity(chunk.len() * cols_count);

            for row in chunk {
                for (_, val) in row.to_values()? {
                    params.push(val);
                }
            }
            chunks.push(Chunk { base_sql, params });
        }

        Ok(chunks)
    }
}

// The `execute` contains returning data
impl<'a, T: Model, O> Insert<'a, T, Returning<O>> {
    /// Executes the insert and parses the returned dataset.
    pub fn execute(self) -> Result<Vec<O>> {
        let chunks = self.generate_chunks()?;
        execute_returning(self.executor, chunks, &self.returning.projection, self.returning.parser)
    }
}

/// Represents an INSERT ... ON CONFLICT builder (Upsert)
/// Shares the returning state `S` with the base `Insert` builder.
pub struct InsertUpdate<'a, T, S = NoReturning> {
    builder: Insert<'a, T, S>,
    conflict_cols: Vec<&'static str>,
    assignments: Vec<Assignment>,
    condition: Option<Condition>,
}

impl<'a, T: Model, S> InsertUpdate<'a, T, S> {
    /// Adds a `SET col = val` or `SET col = excluded.col` assignment
    pub fn update(mut self, a: Assignment) -> Self {
        self.assignments.push(a);
        self
    }

    /// Adds a `WHERE` filter for the `DO UPDATE SET` action
    /// Calling this multiple times will automatically combine them with `AND`.
    pub fn filter(mut self, cond: Condition) -> Self {
        self.condition = match self.condition {
            Some(existing) => Some(existing & cond),
            None => Some(cond),
        };
        self
    }

    /// Generates base chunks and appends the `ON CONFLICT` clause to each of them.
    fn generate_upsert_chunks(&self) -> Result<Vec<Chunk>> {
        let (suffix_sql, suffix_params) = self.build_upsert_suffix()?;
        let mut chunks = self.builder.generate_chunks()?;

        for chunk in &mut chunks {
            chunk.base_sql.push_str(&suffix_sql);
            chunk.params.extend(suffix_params.clone());
        }

        Ok(chunks)
    }

    /// Compiles the `ON CONFLICT` clause and appends it to the base query
    fn build_upsert_suffix(&self) -> Result<(String, Vec<Value>)> {
        let mut sql = String::from(" on conflict (");
        sql.push_str(&self.conflict_cols.join(", "));
        sql.push_str(") do ");

        let mut params = Vec::new();

        if self.assignments.is_empty() {
            sql.push_str("nothing");
        } else {
            sql.push_str("update set ");

            build_assignments(&self.assignments, &mut sql, &mut params)?;

            if let Some(cond) = &self.condition {
                sql.push_str(" where ");
                // Instruct the Condition AST to use the "excluded." prefix if necessary
                cond.clone().build(&mut sql, &mut params, QueryContext::InsertUpsert)?;
            }
        }

        Ok((sql, params))
    }
}

impl<'a, T: Model> InsertUpdate<'a, T, NoReturning> {
    /// State transition: Specifies single or multiple columns to return.
    pub fn returning<C: ReturningColumns>(self, cols: C) -> InsertUpdate<'a, T, Returning<C::Output>> {
        InsertUpdate {
            builder: self.builder.returning(cols), // Delegate to the inner builder
            conflict_cols: self.conflict_cols,
            assignments: self.assignments,
            condition: self.condition,
        }
    }

    /// State transition: Specifies a DTO/Model to map the returned row into.
    pub fn returning_as<U: FromRow>(self) -> InsertUpdate<'a, T, Returning<U>> {
        InsertUpdate {
            builder: self.builder.returning_as::<U>(), // Delegate to the inner builder
            conflict_cols: self.conflict_cols,
            assignments: self.assignments,
            condition: self.condition,
        }
    }

    /// Executes the upsert statement, returning the total number of affected rows.
    pub fn execute(self) -> Result<usize> {
        let chunks = self.generate_upsert_chunks()?;
        execute_affected(self.builder.executor, chunks)
    }
}

impl<'a, T: Model, O> InsertUpdate<'a, T, Returning<O>> {
    /// Executes the upsert and parses the returned dataset.
    pub fn execute(self) -> Result<Vec<O>> {
        let chunks = self.generate_upsert_chunks()?;
        execute_returning(
            self.builder.executor,
            chunks,
            &self.builder.returning.projection,
            self.builder.returning.parser
        )
    }
}

// --- Shared Execution Helpers ---

struct Chunk {
    base_sql: String,
    params: Vec<Value>,
}

/// Helper function to execute chunks without returning data (Result<usize>)
fn execute_affected(executor: Executor<'_>, chunks: Vec<Chunk>) -> Result<usize> {
    executor.with_conn(|conn| {
        let mut total_affected = 0;
        for chunk in chunks {
            total_affected += conn.execute(&chunk.base_sql, params_from_iter(chunk.params))?;
        }
        Ok(total_affected)    
    })
    
}

/// Helper function to execute chunks and parse returning data into type `O`
fn execute_returning<O>(
    executor: Executor<'_>,
    chunks: Vec<Chunk>,
    projection: &str,
    parser: fn(&rusqlite::Row) -> Result<O>
) -> Result<Vec<O>> {
    executor.with_conn(|conn| {
        let mut result = Vec::new();
        let returning_suffix = format!(" returning {}", projection);
        for mut chunk in chunks {
            chunk.base_sql.push_str(&returning_suffix);
            let mut stmt = conn.prepare(&chunk.base_sql)?;
            let mut rows = stmt.query(params_from_iter(chunk.params))?;

            while let Some(row) = rows.next()? {
                result.push(parser(row)?);
            }
        }
        Ok(result)    
    })
}

fn build_base_insert(pattern: Pattern, table: &str, cols: &[&str], rows_count: usize) -> String {
    let mut sql = String::new();
    match pattern {
        Pattern::Normal => sql.push_str("insert into "),
        Pattern::OrIgnore => sql.push_str("insert or ignore into "),
        Pattern::OrReplace => sql.push_str("insert or replace into "),
    }
    sql.push_str(table);

    if cols.is_empty() || rows_count == 0 {
        sql.push_str(" default values");
        return sql;
    }

    sql.push_str(" (");
    sql.push_str(&cols.join(", "));
    sql.push_str(") values ");

    let placeholders = format!("({})", vec!["?"; cols.len()].join(", "));
    let all_placeholders = vec![placeholders.as_str(); rows_count].join(", ");
    sql.push_str(&all_placeholders);

    sql
}

#[derive(Copy, Clone)]
pub(crate) enum Pattern {
    Normal,
    OrIgnore,
    OrReplace,
}

/// A trait used to normalize single structs, arrays, or vectors into a slice.
pub(crate) trait AsRows<'a, T> {
    fn as_rows(&self) -> &'a [T];
}

impl<'a, T> AsRows<'a, T> for &'a T {
    fn as_rows(&self) -> &'a [T] {
        std::slice::from_ref(self)
    }
}

impl<'a, T> AsRows<'a, T> for &'a [T] {
    fn as_rows(&self) -> &'a [T] {
        self
    }
}

impl<'a, T> AsRows<'a, T> for &'a Vec<T> {
    fn as_rows(&self) -> &'a [T] {
        self
    }
}

/// Defines valid targets for the `ON CONFLICT` clause.
pub(crate) trait ConflictTarget {
    fn to_columns(&self) -> Vec<&'static str>;
}

// implementation for single `Column<T>`
impl<T> ConflictTarget for Column<T> {
    fn to_columns(&self) -> Vec<&'static str> {
        vec![self.meta.column_name]
    }
}

// Automatically implement Traits for multi-column combinations (Tuples) with macros
// that erase their type differences
macro_rules! impl_conflict_target_for_tuple {
    ( $($T:ident),+ ) => {
        // The `$T` here represents any type that implements `ConflictTarget`
        impl<$($T: ConflictTarget),+> ConflictTarget for ($($T,)+) {
            fn to_columns(&self) -> Vec<&'static str> {
                #[allow(non_snake_case)]
                let ($($T,)+) = self; // Deconstruct Tuple
                let mut cols = Vec::new();
                $( cols.extend($T.to_columns()); )+ // Stuff all the names into the same Vec
                cols
            }
        }
    };
}

// Supports 2-column union indexes
impl_conflict_target_for_tuple!(A, B);
// Supports 3-column union indexes
impl_conflict_target_for_tuple!(A, B, C);
// Supports 4-column union indexes
impl_conflict_target_for_tuple!(A, B, C, D);