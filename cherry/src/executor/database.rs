use crate::executor::transaction::Transaction;
use crate::executor::Executor;
use crate::ops::insert::{AsRows, Insert};
use crate::ops::select::Select;
use crate::types::model::Model;
use crate::Result;
use rusqlite::Connection;
use std::path::Path;
use crate::ops::update::Update;
use crate::ops::delete::Delete;

pub struct Database {
    conn: Connection,
}

impl Database {

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(Self {
            conn: Connection::open(path)?,
        })
    }

    pub fn insert<'a, T: Model, M: AsRows<'a, T>>(&'a self, m: M) -> Insert<'a, T> {
        Insert::create(Executor::Connection(&self.conn), m)
    }

    pub fn select<T: Model>(&'_ self) -> Select<'_, T> {
        Select::create(Executor::Connection(&self.conn))
    }

    pub fn update<T: Model>(&'_ self) -> Update<'_, T> {
        Update::create(Executor::Connection(&self.conn))
    }

    pub fn delete<T: Model>(&'_ self) -> Delete<'_, T> {
        Delete::create(Executor::Connection(&self.conn))
    }

    pub fn transaction<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&Transaction)
            -> Result<()> {

        let inner_tx = self.conn.transaction()?;
        let tx = Transaction::create(Executor::Transaction(&inner_tx));

        f(&tx)?;

        Ok(inner_tx.commit()?)
    }

    // pub fn execute_sql(sql: &str) -> () {
    //
    // }

}
