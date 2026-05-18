use thiserror::Error;

/// Errors that can occur during Cherry database operations.
#[derive(Error, Debug)]
pub enum Error {
    // ==========================================
    // 1. 底层设施错误 (Infrastructure Errors)
    // ==========================================

    /// Wraps errors returned directly from the underlying rusqlite library.
    #[error("database internal error: {0}")]
    Database(#[from] rusqlite::Error),

    /// Represents I/O failures, such as a missing database file.
    #[error("file system error: {0}")]
    Io(#[from] std::io::Error),

    /// Errors related to connection pool exhaustion or initialization failures.
    #[error("connection error: {0}")]
    ConnectionError(String),

    // ==========================================
    // 2. 类型映射与解析错误 (Mapping & Parsing Errors)
    // ==========================================

    /// Occurs when a requested column index or name does not exist in the returned row.
    /// Crucial for dynamic tuple projections and JOIN offset extractions.
    #[error("column not found: {0}")]
    ColumnNotFound(String),

    /// Occurs when the database returns a storage class different from what the Rust type expects.
    /// Extremely useful for custom `Type` implementations during `from_db`.
    #[error("type mismatch: {0}")]
    TypeMismatch(String),

    /// Occurs when failing to parse or deserialize database text/blob into a strongly-typed Rust struct or enum.
    #[error("parse error: {0}")]
    ParseError(String),

    /// Occurs when failing to serialize or convert a Rust type into a database-supported value during `to_db`.
    #[error("conversion/serialization error: {0}")]
    ConversionError(String),

    // ==========================================
    // 3. 业务与查询逻辑错误 (Business & Query Errors)
    // ==========================================

    /// Indicates that a requested record does not exist in the database.
    /// Uses String to support various Primary Key types (i32, i64, UUID, String).
    #[error("record not found (ID: {0})")]
    NotFound(String),

    /// Used when a provided parameter fails structural validation before querying.
    #[error("invalid parameter: {0}")]
    InvalidParam(String),

    /// Used for custom domain-driven validation rules before saving to the database.
    /// E.g., intercepting empty strings or negative balances inside `to_db`.
    #[error("validation error: {0}")]
    ValidationError(String),

    /// Occurs when the AST builder fails to compile into a valid SQL.
    /// E.g., Executing an UPDATE builder without calling `.set()`.
    #[error("query syntax/build error: {0}")]
    QueryError(String),
}