pub mod transaction;
pub mod database;

#[derive(Copy, Clone)]
pub enum Executor<'a> {
    Connection(&'a rusqlite::Connection),
    Transaction(&'a rusqlite::Transaction<'a>),
}

impl<'a> Executor<'a> {
    /// Get a unified underlying Connection reference 
    /// to smooth out the differences between connections and transactions.
    pub fn conn(&self) -> &'a rusqlite::Connection {
        match self {
            Executor::Connection(c) => c,
            // Because of `rusqlite::Transaction` implemented `Deref<Target = Connection>`.
            // The `t` here is automatically dereferenced by the Rust compiler to `&rusqlite::Connection`.
            Executor::Transaction(t) => t,
        }
    }
}