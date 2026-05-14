use crate::executor::Executor;
use crate::model::{FromRow, Row};
use crate::ops::dsl::assignment::{build_assignments, Assignment};
use crate::ops::dsl::column::Column;
use crate::ops::dsl::QueryContext;
use crate::query::dsl::Condition;
use crate::types::model::Model;
use crate::{DbType, DbValue, Error, Result};
use rusqlite::params_from_iter;

/// Represents the builder for an INSERT query.
pub struct Insert<'a, T> {
    pub(crate) executor: Executor<'a>,
    pub(crate) rows: &'a [T],
    pub(crate) pattern: Pattern,
}

impl<'a, T: Model> Insert<'a, T> {

    pub(crate) fn create<M>(executor: Executor<'a>, m: M) -> Insert<'a, T>
    where
        M: AsRows<'a, T>,
    {
        Insert {
            executor,
            rows: m.as_rows(),
            pattern: Pattern::Normal,
        }
    }

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

    /// Appends the `on conflict` clause, returning an InsertUpdate builder
    /// which allows chaining `update` (for upsert) and `filter`.
    pub fn on_conflict<Target: ConflictTarget>(self, target: Target) -> InsertUpdate<'a, T> {
        InsertUpdate {
            builder: self,
            conflict_cols: target.to_columns(),
            assignments: vec![],
            condition: None,
        }
    }

    /// Executes the insert statement, returning the total number of affected rows.
    /// Automatically chunks the execution to avoid SQLite's 32766 parameter limit.
    pub fn execute(self) -> Result<usize> {
        let chunks = self.generate_chunks();
        let conn = self.executor.conn();
        let mut total_affected = 0;

        for chunk in chunks {
            total_affected += conn.execute(&chunk.base_sql, params_from_iter(chunk.params))?;
        }

        Ok(total_affected)
    }

    /// Executes the insert and returns a single column from the inserted rows.
    /// Useful for fetching auto-incremented IDs (`returning(User::ID)`).
    pub fn returning<U: DbType>(self, col: Column<U>) -> Result<Vec<U>> {
        let chunks = self.generate_chunks();
        let returning_suffix = format!(" returning {}", col.name);
        execute_returning_column(self.executor, chunks, &returning_suffix)
    }

    /// Executes the insert and projects the returned rows into a specified DTO.
    /// The target type `U` must implement `FromRow`.
    pub fn returning_as<U: FromRow>(self) -> Result<Vec<U>> {
        let chunks = self.generate_chunks();
        let cols = U::columns().iter().map(|c| c.name).collect::<Vec<_>>().join(", ");
        let returning_suffix = format!(" returning {}", cols);
        execute_returning_as(self.executor, chunks, &returning_suffix)
    }

    // --- Private Helpers ---

    /// Generates executable chunks of SQL and parameters to bypass SQLite limits.
    fn generate_chunks(&self) -> Vec<ChunkData> {
        let mut chunks = Vec::new();

        if self.rows.is_empty() {
            chunks.push(ChunkData {
                base_sql: build_base_insert(self.pattern, T::table_name(), &[], 0),
                params: vec![],
            });
            return chunks;
        }

        let first_vals = self.rows[0].to_values();
        let col_names: Vec<&str> = first_vals.iter().map(|(k, _)| *k).collect();
        let cols_count = col_names.len();

        if cols_count == 0 {
            let sql = build_base_insert(self.pattern, T::table_name(), &[], 0);
            for _ in self.rows {
                chunks.push(ChunkData { base_sql: sql.clone(), params: vec![] });
            }
            return chunks;
        }

        // SQLite limit is 32766. We leave a buffer of 200 for Upsert/WHERE parameters.
        let max_params = 32566;
        let chunk_size = std::cmp::max(1, max_params / cols_count);

        for chunk in self.rows.chunks(chunk_size) {
            let base_sql = build_base_insert(self.pattern, T::table_name(), &col_names, chunk.len());
            let mut params = Vec::with_capacity(chunk.len() * cols_count);

            for row in chunk {
                for (_, val) in row.to_values() {
                    params.push(val);
                }
            }
            chunks.push(ChunkData { base_sql, params });
        }

        chunks
    }
}

// --- Helper Structs & Functions ---

struct ChunkData {
    base_sql: String,
    params: Vec<DbValue>,
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

/// Helper function to execute chunks and return a single column
fn execute_returning_column<U: DbType>(executor: Executor<'_>, chunks: Vec<ChunkData>, returning_suffix: &str)
                                       -> Result<Vec<U>> {
    let conn = executor.conn();
    let mut result = Vec::new();

    for mut chunk in chunks {
        chunk.base_sql.push_str(returning_suffix);
        let mut stmt = conn.prepare(&chunk.base_sql)?;
        let mut rows = stmt.query(params_from_iter(chunk.params))?;

        while let Some(row) = rows.next()? {
            let db_val: DbValue = row.get(0)
                .map_err(|e| Error::QueryError(e.to_string()))?;
            let val = U::from_db_value(db_val)?;
            result.push(val);
        }
    }
    Ok(result)
}

/// Helper function to execute chunks and project returned rows into a DTO
fn execute_returning_as<U: FromRow>(executor: Executor<'_>, chunks: Vec<ChunkData>, returning_suffix: &str)
                                    -> Result<Vec<U>> {
    let conn = executor.conn();
    let mut result = Vec::new();

    for mut chunk in chunks {
        chunk.base_sql.push_str(returning_suffix);
        let mut stmt = conn.prepare(&chunk.base_sql)?;
        let mut rows = stmt.query(params_from_iter(chunk.params))?;

        while let Some(row) = rows.next()? {
            result.push(U::from_row(&Row::new(row))?);
        }
    }
    Ok(result)
}

/// Represents an INSERT ... ON CONFLICT builder (Upsert)
pub struct InsertUpdate<'a, T> {
    builder: Insert<'a, T>,
    conflict_cols: Vec<&'static str>,
    assignments: Vec<Assignment>,
    condition: Option<Condition>,
}

impl<'a, T: Model> InsertUpdate<'a, T> {

    /// Adds a `SET col = val` or `SET col = excluded.col` assignment
    pub fn update(mut self, a: Assignment) -> Self {
        self.assignments.push(a);
        self
    }

    /// Adds a `WHERE` filter for the `DO UPDATE SET` action
    pub fn filter(mut self, cond: Condition) -> Self {
        self.condition = Some(cond);
        self
    }

    /// Executes the upsert statement
    pub fn execute(self) -> Result<usize> {
        let chunks = self.generate_upsert_chunks();
        let conn = self.builder.executor.conn();
        let mut total_affected = 0;

        for chunk in chunks {
            total_affected += conn.execute(&chunk.base_sql, params_from_iter(chunk.params))?;
        }

        Ok(total_affected)
    }

    /// Executes the upsert and returns a single column from the updated/inserted rows.
    pub fn returning<U: DbType>(self, col: Column<U>) -> Result<Vec<U>> {
        let chunks = self.generate_upsert_chunks();
        let returning_suffix = format!(" returning {}", col.name);
        execute_returning_column(self.builder.executor, chunks, &returning_suffix)
    }

    /// Executes the upsert and projects the returned rows into a specified DTO.
    pub fn returning_as<U: FromRow>(self) -> Result<Vec<U>> {
        let chunks = self.generate_upsert_chunks();
        let cols = U::columns().iter().map(|c| c.name).collect::<Vec<_>>().join(", ");
        let returning_suffix = format!(" returning {}", cols);
        execute_returning_as(self.builder.executor, chunks, &returning_suffix)
    }

    // --- Private Helpers ---

    /// Generates base chunks and appends the `ON CONFLICT` clause to each of them.
    fn generate_upsert_chunks(&self) -> Vec<ChunkData> {
        let (suffix_sql, suffix_params) = self.build_upsert_suffix();
        let mut chunks = self.builder.generate_chunks();

        for chunk in &mut chunks {
            chunk.base_sql.push_str(&suffix_sql);
            chunk.params.extend(suffix_params.clone());
        }

        chunks
    }

    /// Compiles the `ON CONFLICT` clause and appends it to the base query
    fn build_upsert_suffix(&self) -> (String, Vec<DbValue>) {
        let mut sql = String::from(" on conflict (");
        sql.push_str(&self.conflict_cols.join(", "));
        sql.push_str(") do ");

        let mut params = Vec::new();

        if self.assignments.is_empty() {
            sql.push_str("nothing");
        } else {
            sql.push_str("update set ");

            build_assignments(&self.assignments, &mut sql, &mut params);

            if let Some(cond) = &self.condition {
                sql.push_str(" where ");
                // Instruct the Condition AST to use the "excluded." prefix if necessary
                cond.clone().build(&mut sql, &mut params, QueryContext::InsertUpsert);
            }
        }

        (sql, params)
    }
}

#[derive(Copy, Clone)]
pub(crate) enum Pattern {
    Normal,
    OrIgnore,
    OrReplace,
}

/// A trait used to normalize single structs, arrays, or vectors into a slice.
pub trait AsRows<'a, T> {
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
pub trait ConflictTarget {
    fn to_columns(&self) -> Vec<&'static str>;
}

// implementation for single `Column<T>`
impl<T> ConflictTarget for Column<T> {
    fn to_columns(&self) -> Vec<&'static str> {
        vec![self.name]
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