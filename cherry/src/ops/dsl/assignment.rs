use crate::core::model::ColumnMeta;
use crate::core::value::{AssignExpr, Value};
use crate::{Error, Result};

/// Represents an assignment operation in an `UPDATE` or `ON CONFLICT DO UPDATE` (Upsert) statement.
#[derive(Debug, Clone)]
pub enum Assignment {
    /// Standard value assignment.
    /// Generates: `col = ?`
    Value(ColumnMeta, AssignExpr),

    /// Increment (or decrement if the value is negative) assignment.
    /// Generates: `col = col + ?`
    Increment(ColumnMeta, AssignExpr),

    /// Upsert conflict resolution leveraging the `excluded` table.
    /// Generates: `col = excluded.col` (No parameter binding required)
    Excluded(ColumnMeta),
}

/// Builds the SQL string and binds parameters for a slice of assignments.
pub fn build_assignments(assignments: &[Assignment], sql: &mut String, params: &mut Vec<Value>) -> Result<()> {
    let mut writer = AssignmentWriter { sql, params };
    writer.write_assignments(assignments)
}

// ==========================================
// Core Helpers (Writer Pattern)
// ==========================================

/// Encapsulates the SQL building context for assignments, maintaining consistency
/// with the `SqlWriter` pattern used in `condition.rs`.
struct AssignmentWriter<'a> {
    sql: &'a mut String,
    params: &'a mut Vec<Value>,
}

impl<'a> AssignmentWriter<'a> {
    /// Iterates over assignments and joins them with commas.
    fn write_assignments(&mut self, assignments: &[Assignment]) -> Result<()> {
        for (i, assign) in assignments.iter().enumerate() {
            if i > 0 {
                self.sql.push_str(", ");
            }
            self.write_single(assign)?;
        }
        Ok(())
    }

    /// Resolves a single assignment into its corresponding SQL string and parameter bindings.
    fn write_single(&mut self, assign: &Assignment) -> Result<()> {
        match assign {
            Assignment::Value(col, expr) => {
                // e.g., column_name = ?
                self.sql.push_str(col.column_name);
                self.sql.push_str(" = ?");
                self.params.push(Self::take_value(expr.clone())?);
            }
            Assignment::Increment(col, expr) => {
                // e.g., column_name = column_name + ?
                self.sql.push_str(col.column_name);
                self.sql.push_str(" = ");
                self.sql.push_str(col.column_name);
                self.sql.push_str(" + ?");
                self.params.push(Self::take_value(expr.clone())?);
            }
            Assignment::Excluded(col) => {
                // e.g., column_name = excluded.column_name
                self.sql.push_str(col.column_name);
                self.sql.push_str(" = excluded.");
                self.sql.push_str(col.column_name);
            }
        }
        Ok(())
    }

    fn take_value(expr: AssignExpr) -> Result<Value> {
        match expr {
            AssignExpr::Value(v) => Ok(v),
            AssignExpr::Error(e) => Err(Error::ConversionError(e)),
        }
    }
}