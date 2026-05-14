use crate::ops::dsl::assignment::Assignment;
use crate::ops::dsl::condition::Condition;
use crate::types::value::DbType;
use std::marker::PhantomData;

/// Database column definitions
pub struct Column<T> {
    pub(crate) name: &'static str,
    _marker: PhantomData<T>,
}

/// Unconstrained impl blocks, specialized for `const fn` constructors
impl<T> Column<T> {
    pub const fn new(name: &'static str) -> Self {
        Self { name, _marker: PhantomData }
    }
}

// An impl block with trait constraints, containing all query DSL methods
impl<T: DbType> Column<T> {

    pub fn eq(&self, value: impl Into<T>) -> Condition {
        Condition::Eq(self.name, value.into().to_db_value())
    }

    pub fn not_eq(&self, value: impl Into<T>) -> Condition {
        Condition::NotEq(self.name, value.into().to_db_value())
    }

    pub fn gt(&self, value: impl Into<T>) -> Condition {
        Condition::Gt(self.name, value.into().to_db_value())
    }

    pub fn gte(&self, value: impl Into<T>) -> Condition {
        Condition::Gte(self.name, value.into().to_db_value())
    }

    pub fn lt(&self, value: impl Into<T>) -> Condition {
        Condition::Lt(self.name, value.into().to_db_value())
    }

    pub fn lte(&self, value: impl Into<T>) -> Condition {
        Condition::Lte(self.name, value.into().to_db_value())
    }

    pub fn is_null(&self) -> Condition {
        Condition::IsNull(self.name)
    }

    pub fn is_not_null(&self) -> Condition {
        Condition::IsNotNull(self.name)
    }

    pub fn in_list(&self, values: Vec<impl Into<T>>) -> Condition {
        let db_values = values.into_iter().map(|v| v.into().to_db_value()).collect();
        Condition::In(self.name, db_values)
    }

    pub fn not_list(&self, values: Vec<impl Into<T>>) -> Condition {
        let db_values = values.into_iter().map(|v| v.into().to_db_value()).collect();
        Condition::NotIn(self.name, db_values)
    }

    pub fn between(&self, lower: impl Into<T>, upper: impl Into<T>) -> Condition {
        Condition::Between(self.name, lower.into().to_db_value(),  upper.into().to_db_value())
    }

    pub fn like(&self, value: impl Into<T>) -> Condition {
        Condition::Like(self.name, value.into().to_db_value())
    }

    pub fn starts_with(&self, value: impl Into<T>) -> Condition {
        Condition::StartsWith(self.name, value.into().to_db_value())
    }

    pub fn ends_with(&self, value: impl Into<T>) -> Condition {
        Condition::EndsWith(self.name, value.into().to_db_value())
    }

    pub fn contains(&self, value: impl Into<T>) -> Condition {
        Condition::Contains(self.name, value.into().to_db_value())
    }

    /// For regular `update`, such as AGE.set(18)
    pub fn set(&self, value: impl Into<T>) -> Assignment {
        Assignment::Value(self.name, value.into().to_db_value())
    }

    /// For regular `update` increment/decrement, such as `AGE.increment(1)` or `AGE.increment(-1)`
    pub fn increment(&self, value: impl Into<T>) -> Assignment {
        Assignment::Increment(self.name, value.into().to_db_value())
    }

    /// Special for `on conflict do update set`, such as `AGE.excluded()`
    pub fn excluded(&self) -> Assignment {
        Assignment::Excluded(self.name)
    }

    /// Special for `on conflict do update set`, such as `AGE.excluded_increment(1)` or `AGE.excluded_increment(-1)`
    pub fn excluded_increment(&self, value: impl Into<T>) -> Assignment {
        Assignment::ExcludedIncrement(self.name, value.into().to_db_value())
    }
}
