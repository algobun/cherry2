use crate::executor::transaction::Transaction;
use crate::executor::Executor;
use crate::model::Model;
use crate::query::insert::InsertBuilder;
use crate::query::select::SelectBuilder;
use crate::Result;
use rusqlite::Connection;
use std::path::Path;

pub struct Database {
    conn: Connection,
}

impl Database {

    // pub const ABC: String = String::new();

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(Self {
            conn: Connection::open(path)?,
        })
    }

    pub fn insert<'a, T: Model>(&'a self, t: &'a T) -> InsertBuilder<'a, T> {
        InsertBuilder::create(Executor::Connection(&self.conn), t)
    }

    pub fn select<T: Model>(&'_ self) -> SelectBuilder<'_, T> {
        SelectBuilder::create(Executor::Connection(&self.conn))
    }

    pub fn transaction_execute<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&Transaction)
            -> Result<()> {

        let inner_tx = self.conn.transaction()?;
        let tx = Transaction::create(Executor::Connection(&inner_tx));

        f(&tx)?;

        Ok(inner_tx.commit()?)
    }


}

fn a() {
    // let a = Database::ABC.push_str("ab");
}