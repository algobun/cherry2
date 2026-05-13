use crate::ops::dsl::assignment::Assignment;
use crate::ops::dsl::condition::Condition;
use crate::types::value::DbType;
use std::marker::PhantomData;

/// Database column definitions
pub struct Column<T> {
    pub(crate) name: &'static str,
    _marker: PhantomData<T>,
}

// 无约束的 impl 块，专供 const fn 构造函数使用
impl<T> Column<T> {
    pub const fn new(name: &'static str) -> Self {
        Self { name, _marker: PhantomData }
    }
}

// 带有 trait 约束的 impl 块，包含所有的查询 DSL 方法
impl<T: DbType> Column<T> {
    // pub const fn new(name: &'static str) -> Self {
    //     Self { name, _marker: PhantomData }
    // }

    pub fn eq(&self, value: impl Into<T>) -> Condition {
        Condition::Eq(self.name, value.into().to_db_value())
    }

    pub fn not_eq(&self, value: impl Into<T>) -> Condition {
        Condition::NotEq(self.name, value.into().to_db_value())
    }

    pub fn gt(&self, value: impl Into<T>) -> Condition {
        Condition::Gt(self.name, value.into().to_db_value())
    }

    pub fn is_null(&self) -> Condition {
        Condition::IsNull(self.name)
    }

    pub fn in_list(&self, values: Vec<impl Into<T>>) -> Condition {
        let db_values = values.into_iter().map(|v| v.into().to_db_value()).collect();
        Condition::In(self.name, db_values)
    }

    // Update assignment similarly, no longer pre-paste in advance.
    pub fn set(&self, value: impl Into<T>) -> Assignment {
        Assignment {
            column: self.name,
            value: value.into().to_db_value(),
        }
    }
}
