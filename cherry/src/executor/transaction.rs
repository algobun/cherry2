use crate::executor::Executor;
use crate::ops::delete::Delete;
use crate::ops::insert::AsRows;
use crate::ops::insert::Insert;
use crate::ops::select::Select;
use crate::query::update::Update;
use crate::types::model::Model;

pub struct Transaction<'a> {
    executor: Executor<'a>
}

impl<'a> Transaction<'a> {

    pub(crate) fn create(executor: Executor<'a>) -> Transaction<'a> {
        Transaction { executor }
    }

    pub fn insert<T: Model, M: AsRows<'a, T>>(&'a self, m: M) -> Insert<'a, T> {
        Insert::create(self.executor, m)
    }

    pub fn select<T: Model>(&'_ self) -> Select<'_, T> {
        Select::create(self.executor)
    }

    pub fn update<T: Model>(&'a self) -> Update<'a, T> {
        Update::create(self.executor)
    }

    pub fn delete<T: Model>(&'a self) -> Delete<'a, T> {
        Delete::create(self.executor)
    }
    
}
