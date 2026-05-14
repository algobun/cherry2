use crate::executor::transaction::Transaction;
use crate::executor::Executor;
use crate::ops::insert::Insert;
use crate::ops::select::Select;
use crate::types::model::Model;
use crate::Result;
use rusqlite::Connection;
use std::path::Path;

pub struct Database {
    conn: Connection,
}

impl Database {

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(Self {
            conn: Connection::open(path)?,
        })
    }

    pub fn insert<'a, T: Model>(&'a self, t: &'a T) -> Insert<'a, T> {
        Insert::create(Executor::Connection(&self.conn), t)
    }

    pub fn select<T: Model>(&'_ self) -> Select<'_, T> {
        Select::create(Executor::Connection(&self.conn))
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
