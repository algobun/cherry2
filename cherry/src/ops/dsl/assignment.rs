use crate::core::column::expr::SqlExpr;
use crate::core::model::ColumnMeta;
use crate::core::value::Value;
use crate::{Error, Result};

/// Represents an assignment operation in an `UPDATE` or `ON CONFLICT DO UPDATE` (Upsert) statement.
#[derive(Debug, Clone)]
pub enum Assignment {
    /// Standard value assignment.
    /// Generates: `col = ?` or `col = other_col`
    Value(ColumnMeta, SqlExpr),

    /// Increment (or decrement if the value is negative) assignment.
    /// Generates: `col = col + ?` or `col = col + other_col`
    Increment(ColumnMeta, SqlExpr),

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
                // e.g., column_name = 
                self.sql.push_str(col.column_name);
                self.sql.push_str(" = ");
                self.write_expr(expr)?;
            }
            Assignment::Increment(col, expr) => {
                // e.g., column_name = column_name + 
                self.sql.push_str(col.column_name);
                self.sql.push_str(" = ");
                self.sql.push_str(col.column_name);
                self.sql.push_str(" + ");
                self.write_expr(expr)?;
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

    /// Safely evaluates the unified `CondExpr` expression tree.
    /// This gracefully handles literal values, column references, and poisoned error states.
    fn write_expr(&mut self, expr: &SqlExpr) -> Result<()> {
        match expr {
            SqlExpr::Column(c) => {
                // For column-to-column assignments (e.g., `updated_at = created_at`)
                // Directly append the target column's name without binding any parameters.
                self.sql.push_str(c.column_name);
                Ok(())
            }
            SqlExpr::Value(v) => {
                // For literal assignments (e.g., `age = ?`)
                // Emit a placeholder and push the value into the binding pool.
                self.sql.push('?');
                self.params.push(v.clone());
                Ok(())
            }
            SqlExpr::Error(e) => {
                // Intercept and bubble up any serialization/conversion failures
                // that occurred during the DSL builder phase.
                Err(Error::ConversionError(e.clone()))
            }
        }
    }
}