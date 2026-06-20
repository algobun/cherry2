use crate::core::column::expr::IntoSqlExpr;
use crate::core::model::ColumnMeta;
use crate::core::value::Type;
use crate::ops::dsl::assignment::Assignment;
use crate::ops::dsl::condition::Condition;
use crate::ops::dsl::condition::Operator;
use crate::ops::dsl::order::{OrderBy, OrderDir};
use std::marker::PhantomData;

/// Database column definitions
pub struct Column<T> {
    pub(crate) meta: ColumnMeta,
    _marker: PhantomData<T>,
}

/// Unconstrained impl blocks, specialized for `const fn` constructors
impl<T> Column<T> {
    pub const fn new(meta: ColumnMeta) -> Self {
        Self { meta, _marker: PhantomData }
    }
}

// An impl block with trait constraints, containing all query DSL methods
impl<T: Type> Column<T> {

    pub fn eq(&self, value: impl IntoSqlExpr<T>) -> Condition {
        Condition::Compare(self.meta, Operator::Eq, value.into_sql_expr())
    }

    pub fn not_eq(&self, value: impl IntoSqlExpr<T>) -> Condition {
        Condition::Compare(self.meta, Operator::NotEq, value.into_sql_expr())
    }

    pub fn gt(&self, value: impl IntoSqlExpr<T>) -> Condition {
        Condition::Compare(self.meta, Operator::Gt, value.into_sql_expr())
    }

    pub fn gte(&self, value: impl IntoSqlExpr<T>) -> Condition {
        Condition::Compare(self.meta, Operator::Gte, value.into_sql_expr())
    }

    pub fn lt(&self, value: impl IntoSqlExpr<T>) -> Condition {
        Condition::Compare(self.meta, Operator::Lt, value.into_sql_expr())
    }

    pub fn lte(&self, value: impl IntoSqlExpr<T>) -> Condition {
        Condition::Compare(self.meta, Operator::Lte, value.into_sql_expr())
    }

    pub fn is_null(&self) -> Condition {
        Condition::IsNull(self.meta)
    }

    pub fn is_not_null(&self) -> Condition {
        Condition::IsNotNull(self.meta)
    }

    pub fn in_list<I, E>(&self, values: I) -> Condition
    where
        I: IntoIterator<Item = E>,
        E: IntoSqlExpr<T> {
        let exprs = values.into_iter().map(|v| v.into_sql_expr()).collect();
        Condition::In(self.meta, exprs)
    }

    pub fn not_list<I, E>(&self, values: I) -> Condition
    where
        I: IntoIterator<Item = E>,
        E: IntoSqlExpr<T> {
        let exprs = values.into_iter().map(|v| v.into_sql_expr()).collect();
        Condition::NotIn(self.meta, exprs)
    }

    pub fn between(&self, lower: impl IntoSqlExpr<T>, upper: impl IntoSqlExpr<T>) -> Condition {
        Condition::Between(self.meta, lower.into_sql_expr(), upper.into_sql_expr())
    }

    pub fn like(&self, value: impl IntoSqlExpr<T>) -> Condition {
        Condition::Like(self.meta, value.into_sql_expr())
    }

    pub fn starts_with(&self, value: impl IntoSqlExpr<T>) -> Condition {
        Condition::StartsWith(self.meta, value.into_sql_expr())
    }

    pub fn ends_with(&self, value: impl IntoSqlExpr<T>) -> Condition {
        Condition::EndsWith(self.meta, value.into_sql_expr())
    }

    pub fn contains(&self, value: impl IntoSqlExpr<T>) -> Condition {
        Condition::Contains(self.meta, value.into_sql_expr())
    }

    pub fn asc(&self) -> OrderBy {
        OrderBy { column: self.meta, direction: OrderDir::Asc }
    }

    pub fn desc(&self) -> OrderBy {
        OrderBy { column: self.meta, direction: OrderDir::Desc }
    }

    /// For regular `update`, such as AGE.set(18)
    pub fn set(&self, value: impl IntoSqlExpr<T>) -> Assignment {
        Assignment::Value(self.meta, value.into_sql_expr())
    }

    /// For regular `update` increment/decrement, such as `AGE.increment(1)` or `AGE.increment(-1)`
    pub fn increment(&self, value: impl IntoSqlExpr<T>) -> Assignment {
        Assignment::Increment(self.meta, value.into_sql_expr())
    }

    /// Special for `on conflict do update set`, such as `AGE.excluded()`
    pub fn excluded(&self) -> Assignment {
        Assignment::Excluded(self.meta)
    }
    
    // /// Special for `on conflict do update set`, such as `AGE.excluded_increment(1)` or `AGE.excluded_increment(-1)`
    // pub fn excluded_increment(&self, value: impl IntoExpr<T>) -> Assignment {
    //     Assignment::ExcludedIncrement(self.meta, value.into().to_db())
    // }
}
