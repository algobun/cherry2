use crate::query::dsl::Column;
use crate::types::{ColumnMeta, Type, Value};
use crate::Result;

/// Represents an evaluated expression within a SQL condition.
/// This enum captures the delayed evaluation state, seamlessly handling
/// columns, literal values, and serialization failures.
#[derive(Debug, Clone)]
pub enum SqlExpr {
    Column(ColumnMeta),
    Value(Value),
    /// Acts as a "poisoned state" node. If a value fails to serialize into a database type,
    /// the error is captured here and bubbles up during the final AST evaluation.
    Error(String),
}

impl From<ColumnMeta> for SqlExpr {
    fn from(value: ColumnMeta) -> Self {
        SqlExpr::Column(value)
    }
}

impl From<Result<Value>> for SqlExpr {
    fn from(value: Result<Value>) -> Self {
        match value {
            Ok(v) => SqlExpr::Value(v),
            Err(e) => SqlExpr::Error(e.to_string()),
        }
    }
}

/// A unified trait for the right-hand side of SQL condition expressions.
///
/// It abstracts away the difference between comparing a column to a literal value
/// (e.g., `WHERE age = 18`) and comparing it to another column (e.g., `JOIN ON a.id = b.id`).
pub trait IntoSqlExpr<T> {
    fn into_sql_expr(self) -> SqlExpr;
}

// ==========================================
// Core Implementations
// ==========================================

/// Resolves column-to-column comparisons (typically used in `JOIN ON` clauses).
///
/// Ensures strict type safety by requiring both columns to share the exact same generic type `T`.
impl<T> IntoSqlExpr<T> for Column<T> {
    fn into_sql_expr(self) -> SqlExpr {
        SqlExpr::Column(self.meta)
    }
}

/// Resolves column-to-value comparisons (typically used in `WHERE` clauses).
///
/// Serializes the given Rust value into a database-compatible `Expr::Value`
/// via the `Type::to_db()` trait bound.
impl<T: Type> IntoSqlExpr<T> for T {
    fn into_sql_expr(self) -> SqlExpr {
        SqlExpr::from(self.to_db())
    }
}

// ==========================================
// Reference Bridges (Zero-Cost Ergonomics)
// ==========================================

/// Bridge for standard references (e.g., `&String`, `&u32`), allowing zero-copy evaluation.
/// Since `Type::to_db` inherently operates on `&self`, this is a natural fit.
impl<'a, T: Type> IntoSqlExpr<T> for &'a T {
    fn into_sql_expr(self) -> SqlExpr {
        SqlExpr::from(self.to_db())
    }
}

/// Ergonomic bridge allowing `Column<String>` to directly accept string slices (`&str`)
/// without requiring manual `.to_string()` allocations from the caller.
impl<'a> IntoSqlExpr<String> for &'a str {
    fn into_sql_expr(self) -> SqlExpr {
        SqlExpr::from(self.to_db())
    }
}

/// Ergonomic bridge allowing `Column<Vec<u8>>` to directly accept byte slices (`&[u8]`)
/// without requiring manual `.to_vec()` allocations from the caller.
impl<'a> IntoSqlExpr<Vec<u8>> for &'a [u8] {
    fn into_sql_expr(self) -> SqlExpr {
        SqlExpr::from(self.to_db())
    }
}