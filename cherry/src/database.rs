use crate::Result;
use rusqlite::Connection;
use std::path::Path;

pub struct Database {
    conn: Connection,
}

impl Database {

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {

        let conn = Connection::open(path)?;

        todo!()
    }


}