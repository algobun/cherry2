pub mod transaction;

use crate::database::db::Backend;
use crate::Result;

#[derive(Copy, Clone)]
pub enum Executor<'a> {
    Backend(&'a Backend),
    Transaction(&'a rusqlite::Transaction<'a>),
}

impl<'a> Executor<'a> {
    pub fn with_conn<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&rusqlite::Connection) -> Result<R>,
    {
        match self {
            Executor::Backend(backend) => {
                match backend {
                    #[cfg(not(feature = "pool"))]
                    Backend::Single(conn) => f(conn),

                    #[cfg(feature = "pool")]
                    Backend::Pooled(pool) => {
                        let conn = pool.get().map_err(|e| crate::Error::ConnectionError(e.to_string()))?;
                        f(&conn)
                    }
                }
            }
            Executor::Transaction(tx) => f(tx)
        }
    }
}
