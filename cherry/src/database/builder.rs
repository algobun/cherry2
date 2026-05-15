use crate::database::db::{Backend, Database};
use crate::Result;
use std::path::{Path, PathBuf};

/// Configuration builder for the Database.
pub struct DatabaseBuilder {
    path: PathBuf,
    /// Collects all initialization PRAGMA/SQL statements to be executed upon connection.
    init_sql: String,

    /// `None` indicates no connection pool will be used.
    /// `Some(size)` enables the pool with the specified maximum connections.
    #[cfg(feature = "pool")]
    max_pool_size: Option<u32>,
}

impl DatabaseBuilder {
    /// Creates a new configuration builder for the specified database path.
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            init_sql: String::new(),

            #[cfg(feature = "pool")]
            max_pool_size: None, // Defaults to single connection even if the pool feature is enabled
        }
    }

    /// Appends custom initialization SQL or PRAGMA statements.
    pub fn add_pragma(mut self, pragma: &str) -> Self {
        self.init_sql.push_str(pragma);
        self.init_sql.push('\n');
        self
    }

    /// Shortcut method: Applies the recommended high-concurrency settings for SQLite.
    pub fn with_default_pragmas(self) -> Self {
        self.add_pragma("PRAGMA journal_mode = WAL;")
            .add_pragma("PRAGMA synchronous = NORMAL;")
            .add_pragma("PRAGMA busy_timeout = 5000;")
            .add_pragma("PRAGMA foreign_keys = ON;")
    }

    /// Enables the connection pool and sets the maximum number of connections.
    /// (Available only when the `pool` feature is enabled).
    #[cfg(feature = "pool")]
    pub fn pooled(mut self, max_size: u32) -> Self {
        self.max_pool_size = Some(max_size);
        self
    }

    /// Consumes the options and builds the final Database instance.
    pub fn build(self) -> Result<Database> {
        #[cfg(feature = "pool")]
        {
            // If the user does not explicitly set the size,
            // we default it to a size=1 connection pool!
            let size = self.max_pool_size.unwrap_or(1);
            let init_sql = self.init_sql.clone();

            let manager = r2d2_sqlite::SqliteConnectionManager::file(&self.path)
                .with_init(move |c| {
                    if !init_sql.is_empty() {
                        c.execute_batch(&init_sql)?;
                    }
                    Ok(())
                });

            let pool = r2d2::Pool::builder()
                .max_size(size)
                .build(manager)
                .map_err(|e| crate::Error::ConnectionError(e.to_string()))?;

            return Ok(Database { backend: Backend::Pooled(pool) });
        }

        #[cfg(not(feature = "pool"))]
        {
            let conn = rusqlite::Connection::open(&self.path)?;
            if !self.init_sql.is_empty() {
                conn.execute_batch(&self.init_sql)?;
            }
            Ok(Database { backend: Backend::Single(conn) })
        }
    }
}