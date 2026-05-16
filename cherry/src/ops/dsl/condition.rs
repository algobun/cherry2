use crate::model::ColumnMeta;
pub use crate::ops::dsl::QueryContext;
use crate::types::value::{DbValue, Expr};
use std::ops::{BitAnd, BitOr};

/// SQL Binary Operators.
#[derive(Debug, Clone, Copy)]
pub enum Operator {
    Eq,
    NotEq,
    Gt,
    Gte,
    Lt,
    Lte,
}

impl Operator {
    /// Resolves the operator into its corresponding SQL string representation.
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Operator::Eq => " = ",
            Operator::NotEq => " != ",
            Operator::Gt => " > ",
            Operator::Gte => " >= ",
            Operator::Lt => " < ",
            Operator::Lte => " <= ",
        }
    }
}

#[derive(Debug, Clone)]
pub enum Condition {
    /// Consolidated binary comparison (e.g., col = expr, col > expr).
    /// Drastically reduces the number of AST variants.
    Compare(ColumnMeta, Operator, Expr),

    IsNull(ColumnMeta),
    IsNotNull(ColumnMeta),

    In(ColumnMeta, Vec<Expr>),
    NotIn(ColumnMeta, Vec<Expr>),
    Between(ColumnMeta, Expr, Expr),

    Like(ColumnMeta, Expr),
    StartsWith(ColumnMeta, Expr),
    EndsWith(ColumnMeta, Expr),
    Contains(ColumnMeta, Expr),

    // Nested logical combinations
    And(Box<Condition>, Box<Condition>),
    Or(Box<Condition>, Box<Condition>),
}

// Gracefully supports `and` and `or` operators.
// e.g., `cond1 & cond2` AND `cond1 | cond2`
impl BitAnd for Condition {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Condition::And(Box::new(self), Box::new(rhs))
    }
}

impl BitOr for Condition {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Condition::Or(Box::new(self), Box::new(rhs))
    }
}

impl Condition {
    /// Recursively traverses the AST to build the SQL expression and bind parameters.
    pub(crate) fn build(self, sql: &mut String, params: &mut Vec<DbValue>, ctx: QueryContext) {
        // Delegate to a dedicated Writer to eliminate redundant parameter passing.
        let mut writer = SqlWriter { sql, params, ctx };
        writer.write_cond(self);
    }
}

// ==========================================
// Core Helpers (Writer Pattern)
// ==========================================

/// Encapsulates SQL building context to vastly improve cohesion.
struct SqlWriter<'a> {
    sql: &'a mut String,
    params: &'a mut Vec<DbValue>,
    ctx: QueryContext,
}

impl<'a> SqlWriter<'a> {
    fn write_cond(&mut self, cond: Condition) {
        match cond {
            // Consolidated standard binary operations
            Condition::Compare(col, op, expr) => self.write_binary(&col, op, expr),

            // Range checks
            Condition::In(col, exprs) => self.write_in(&col, exprs, true),
            Condition::NotIn(col, exprs) => self.write_in(&col, exprs, false),
            Condition::Between(col, val1, val2) => {
                self.write_col(&col);
                self.sql.push_str(" between ");
                self.write_expr(val1);
                self.sql.push_str(" and ");
                self.write_expr(val2);
            }

            // NULL checks
            Condition::IsNull(col) => {
                self.write_col(&col);
                self.sql.push_str(" is null");
            }
            Condition::IsNotNull(col) => {
                self.write_col(&col);
                self.sql.push_str(" is not null");
            }

            // Wildcard matching
            Condition::Like(col, expr) => self.write_wildcard(&col, expr, "", ""),
            Condition::StartsWith(col, expr) => self.write_wildcard(&col, expr, "", "%"),
            Condition::EndsWith(col, expr) => self.write_wildcard(&col, expr, "%", ""),
            Condition::Contains(col, expr) => self.write_wildcard(&col, expr, "%", "%"),

            // Logical tree traversal
            Condition::And(left, right) => self.write_logic(" and ", *left, *right),
            Condition::Or(left, right) => self.write_logic(" or ", *left, *right),
        }
    }

    /// Appends the properly formatted column name based on context 
    /// e.g., Standard vs SELECT vs UPSERT.
    fn write_col(&mut self, col: &ColumnMeta) {
        if self.ctx == QueryContext::InsertUpsert {
            self.sql.push_str("excluded.");
            self.sql.push_str(col.column_name);
        } else if self.ctx == QueryContext::Select {
            self.sql.push_str(col.table_name);
            self.sql.push('.');
            self.sql.push_str(col.column_name);
        } else {
            self.sql.push_str(col.column_name);
        }
    }

    /// Resolves an Expression, securely binding values or formatting column projections.
    fn write_expr(&mut self, expr: Expr) {
        match expr {
            Expr::Column(meta) => self.write_col(&meta),
            Expr::Value(val) => {
                self.sql.push('?');
                self.params.push(val);
            }
        }
    }

    /// Handles standard binary comparison operations.
    fn write_binary(&mut self, col: &ColumnMeta, op: Operator, expr: Expr) {
        self.write_col(col);
        self.sql.push_str(op.as_str());
        self.write_expr(expr);
    }

    /// Handles nested logical combinations with proper grouping.
    fn write_logic(&mut self, op: &str, left: Condition, right: Condition) {
        self.sql.push('(');
        self.write_cond(left);
        self.sql.push_str(op);
        self.write_cond(right);
        self.sql.push(')');
    }

    /// Handles IN and NOT IN clauses, preventing syntax errors on empty lists.
    fn write_in(&mut self, col: &ColumnMeta, exprs: Vec<Expr>, is_in: bool) {
        self.write_col(col);
        if exprs.is_empty() {
            // Failsafe for empty `IN ()` arrays to prevent SQL syntax errors.
            let fallback = if is_in { " in (select 1 where 0)" } else { " not in (select 1 where 0)" };
            self.sql.push_str(fallback);
            return;
        }

        self.sql.push_str(if is_in { " in (" } else { " not in (" });
        for (i, expr) in exprs.into_iter().enumerate() {
            if i > 0 {
                self.sql.push_str(", ");
            }
            self.write_expr(expr);
        }
        self.sql.push(')');
    }

    /// Advanced wildcard handler supporting native SQLite string concatenation.
    fn write_wildcard(&mut self, col: &ColumnMeta, expr: Expr, prefix: &str, suffix: &str) {
        self.write_col(col);
        self.sql.push_str(" like ");

        match expr {
            Expr::Value(DbValue::Text(s)) => {
                self.sql.push('?');
                self.params.push(DbValue::Text(format!("{}{}{}", prefix, s, suffix)));
            }
            Expr::Value(val) => {
                self.sql.push('?');
                self.params.push(val);
            }
            Expr::Column(c) => {
                if !prefix.is_empty() {
                    self.sql.push_str(" '%' || ");
                }
                self.write_col(&c);
                if !suffix.is_empty() {
                    self.sql.push_str(" || '%' ");
                }
            }
        }
    }
}