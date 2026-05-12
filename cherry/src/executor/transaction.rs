use crate::executor::Executor;
use crate::model::Model;
use crate::query::insert::InsertBuilder;
use crate::query::select::SelectBuilder;

pub struct Transaction<'a> {
    executor: Executor<'a>
}

impl<'a> Transaction<'a> {

    pub(crate) fn create(executor: Executor<'a>) -> Transaction<'a> {
        Transaction { executor }
    }
    
    pub fn insert<T: Model>(&'a self, t: &'a T) -> InsertBuilder<'a, T> {
        InsertBuilder::create(self.executor, t)
    }

    pub fn select<T: Model>(&'_ self) -> SelectBuilder<'_, T> {
        SelectBuilder::create(self.executor)
    }
    
    
}
