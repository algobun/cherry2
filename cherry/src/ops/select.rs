use crate::executor::Executor;
use crate::ops::dsl::condition::Condition;
use crate::types::model::Model;
use crate::Result;
use std::marker::PhantomData;

pub struct SelectBuilder<'a, T> {
    executor: Executor<'a>,
    condition: Option<Condition>, // AST
    _marker: PhantomData<T>,
}

impl<'a, T: Model> SelectBuilder<'a, T> {

    pub(crate) fn create(executor: Executor<'a>) -> SelectBuilder<'a, T> {
        Self { executor, condition: None, _marker: PhantomData }
    }

    pub fn filter(mut self, cond: Condition) -> Self {
        self.condition.replace(cond);
        
        // 如果已经有条件了，默认用 AND 连接（或者你可以覆盖它）
        // If there are already conditions, use AND by default (or you can override it).
        // self.condition = match self.condition {
        //     Some(existing) => Some(existing & cond),
        //     None => Some(cond),
        // };
        self
    }

    pub fn execute(self) -> Result<Vec<T>> {
        // 最终在这里把 condition 树递归解析成 "WHERE name = ? AND (age > ? OR status = ?)"
        todo!()
    }

}