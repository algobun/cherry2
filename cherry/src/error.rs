use thiserror::Error;

/// Errors that can occur during Cherry database operations.
#[derive(Error, Debug)]
pub enum Error {
    /// Wraps errors returned directly from the underlying rusqlite library.
    #[error("database internal error: {0}")]
    Database(#[from] rusqlite::Error),

    /// Represents I/O failures, such as a missing database file.
    #[error("file system error: {0}")]
    Io(#[from] std::io::Error),

    #[error("connection error: {0}")]
    ConnectionError(String),

    #[error("type mismatch error: {0}")]
    TypeMismatch(String),


    #[error("parse error: {0}")]
    ParseError(String),

    /// Used when a provided parameter fails business logic validation.
    #[error("invalid parameter: {0}")]
    InvalidParam(String),

    #[error("query error: {0}")]
    QueryError(String),
    
    /// Indicates that a requested record does not exist in the database.
    #[error("record not found (ID: {0})")]
    NotFound(i32),
}
