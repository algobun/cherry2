pub mod transaction;
pub mod database;

#[derive(Copy, Clone)]
pub enum Executor<'a> {
    Connection(&'a rusqlite::Connection),
    Transaction(&'a rusqlite::Transaction<'a>),
}