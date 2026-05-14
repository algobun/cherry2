use crate::executor::Executor;
use crate::ops::insert::Insert;
use crate::ops::select::SelectBuilder;
use crate::types::model::Model;

pub struct Transaction<'a> {
    executor: Executor<'a>
}

impl<'a> Transaction<'a> {

    pub(crate) fn create(executor: Executor<'a>) -> Transaction<'a> {
        Transaction { executor }
    }
    
    pub fn insert<T: Model>(&'a self, t: &'a T) -> Insert<'a, T> {
        Insert::create(self.executor, t)
    }

    pub fn select<T: Model>(&'_ self) -> SelectBuilder<'_, T> {
        SelectBuilder::create(self.executor)
    }
    
    
}
