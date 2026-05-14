pub(crate) use crate::ops::dsl::QueryContext;
use crate::types::value::DbValue;
use std::ops::{BitAnd, BitOr};

#[derive(Debug, Clone)]
pub enum Condition {
    Eq(&'static str, DbValue),
    NotEq(&'static str, DbValue),
    Gt(&'static str, DbValue),
    Gte(&'static str, DbValue),
    Lt(&'static str, DbValue),
    Lte(&'static str, DbValue),

    IsNull(&'static str),
    IsNotNull(&'static str),

    In(&'static str, Vec<DbValue>),
    NotIn(&'static str, Vec<DbValue>),
    Between(&'static str, DbValue, DbValue),

    Like(&'static str, DbValue),
    StartsWith(&'static str, DbValue),
    EndsWith(&'static str, DbValue),
    Contains(&'static str, DbValue),

    // Exists(Box<SelectQuery>),
    // NotExists(Box<SelectQuery>),
    // Raw(String),

    // Nested combination conditions
    And(Box<Condition>, Box<Condition>),
    Or(Box<Condition>, Box<Condition>),
}

// Gracefully supports `and` and `or` operators ( cond1 & cond2 )
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
    /// Recursive traversal of the AST to generate SQL and parameters
    pub(crate) fn build(self, sql: &mut String, params: &mut Vec<DbValue>, ctx: QueryContext) {
        let prefix = if ctx == QueryContext::InsertUpsert { "excluded." } else { "" };
        match self {
            Condition::Eq(col, val) => {
                push_op(sql, prefix, col, " = ?");
                params.push(val);
            }
            Condition::NotEq(col, val) => {
                push_op(sql, prefix, col, " != ?");
                params.push(val);
            }
            Condition::Gt(col, val) => {
                push_op(sql, prefix, col, " > ?");
                params.push(val);
            }
            Condition::Gte(col, val) => {
                push_op(sql, prefix, col, " >= ?");
                params.push(val);
            }
            Condition::Lt(col, val) => {
                push_op(sql, prefix, col, " < ?");
                params.push(val);
            }
            Condition::Lte(col, val) => {
                push_op(sql, prefix, col, " <= ?");
                params.push(val);
            }
            Condition::In(col, values) => build_in(prefix, col, values, true, sql, params),
            Condition::NotIn(col, values) => build_in(prefix, col, values, false, sql, params),
            Condition::Between(col, val1, val2) => {
                push_op(sql, prefix, col, " between ? and ?");
                params.push(val1);
                params.push(val2);
            }

            Condition::IsNull(col) => push_op(sql, prefix, col, " is null"),
            Condition::IsNotNull(col) => push_op(sql, prefix, col, " is not null"),
            
            Condition::Like(col, val) => {
                push_op(sql, prefix, col, " like ?");
                params.push(val);
            }
            Condition::StartsWith(col, val) => {
                push_op(sql, prefix, col, " like ?");
                params.push(format_wildcard(val, "", "%"));
            }
            Condition::EndsWith(col, val) => {
                push_op(sql, prefix, col, " like ?");
                params.push(format_wildcard(val, "%", ""));
            }
            Condition::Contains(col, val) => {
                push_op(sql, prefix, col, " like ?");
                params.push(format_wildcard(val, "%", "%"));
            }

            Condition::And(left, right) => {
                sql.push_str("(");
                left.build(sql, params, ctx);
                sql.push_str(" and ");
                right.build(sql, params, ctx);
                sql.push_str(")");
            }
            Condition::Or(left, right) => {
                sql.push_str("(");
                left.build(sql, params, ctx);
                sql.push_str(" or ");
                right.build(sql, params, ctx);
                sql.push_str(")");
            }
        }
    }
}

fn build_in(prefix: &str, col: &str, values: Vec<DbValue>, is_in: bool, sql: &mut String, params: &mut Vec<DbValue>) {
    push_op(sql, prefix, col, "");
    if values.is_empty() {
        // Prevent SQLite syntax errors caused by generating `col IN ()`.
        // Here we use a condition that is always false.
        sql.push_str(if is_in { " in (select 1 where 0)" } else { " not in (select 1 where 0)" });
    } else {
        sql.push_str(if is_in { " in (" } else { " not in (" });
        let placeholders = vec!["?"; values.len()].join(", ");
        sql.push_str(&placeholders);
        sql.push_str(")");
        params.extend(values);
    }
}

fn format_wildcard(val: DbValue, prefix: &str, suffix: &str) -> DbValue {
    if let DbValue::Text(s) = val {
        DbValue::Text(format!("{}{}{}", prefix, s, suffix))
    } else {
        // If the user passes a non-string type to LIKE, degradation is handled by the database
        val
    }
}

fn push_op(sql: &mut String, prefix: &str, col: &str, op: &str) {
    sql.push_str(prefix);
    sql.push_str(col);
    sql.push_str(op);
}