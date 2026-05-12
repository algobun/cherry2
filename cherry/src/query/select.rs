use crate::executor::Executor;
use crate::model::Model;
use std::marker::PhantomData;

pub struct SelectBuilder<'a, T> {
    executor: Executor<'a>,
    _marker: PhantomData<T>,
}

impl<'a, T: Model> SelectBuilder<'a, T> {
    
    pub(crate) fn create(executor: Executor<'a>) -> SelectBuilder<'a, T> {
        Self { executor, _marker: PhantomData }
    }
    
    
}