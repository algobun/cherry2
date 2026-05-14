use crate::executor::Executor;
use crate::types::model::Model;
use crate::ops::dsl::assignment::Assignment;
use crate::ops::dsl::column::{Column};
use crate::Result;

pub struct Insert<'a, T> {
    executor: Executor<'a>,
    t: &'a T,
    is_ignore: bool,
}

impl<'a, T: Model> Insert<'a, T> {

    pub(crate) fn create(executor: Executor<'a>, t: &'a T) -> Insert<'a, T> {
        Insert { executor, t, is_ignore: false }
    }


    pub fn or_ignore(mut self) -> Self {
        self.is_ignore = true;
        self
    }

    pub fn execute(self) -> Result<usize> {

        todo!()
    }
    
    pub fn on_conflict<C>(self, col: Column<C>) -> InsertUpdate<'a, T, C> {
        InsertUpdate { builder: self, col, assignments: vec![] }
    }

}


// pub struct InsertIgnore<'a, T> {
//     builder: Insert<'a, T>,
// }
// 
// impl<T> InsertIgnore<'_, T> {
// 
//     pub fn execute(self) -> Result<usize> {
// 
//         todo!()
//     }
// }



pub struct InsertUpdate<'a, T, C> {
    builder: Insert<'a, T>,
    col: Column<C>,
    assignments: Vec<Assignment>,
}

impl<'a, T, C> InsertUpdate<'a, T, C> {

    pub fn update(mut self, a: Assignment) -> Self {
        self.assignments.push(a);
        self
    }

    pub fn execute(self) -> Result<usize> {

        todo!()
    }

}
