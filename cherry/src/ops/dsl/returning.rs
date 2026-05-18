use crate::core::model::FromRow;
use crate::core::row::Row;
use crate::core::value::{Type, Value};
use crate::ops::dsl::column::Column;
use crate::{Error, Result};

/// Represents the default state where no data is returned from the query.
pub struct NoReturning;

/// Holds the projection (columns) and the parser function required to
/// extract data from a `returning` clause.
pub struct Returning<O> {
    /// The comma-separated list of columns to return (e.g., "id, name").
    pub projection: String,
    /// A function pointer to parse a single `rusqlite::Row` into the expected output type `O`.
    pub parser: fn(&rusqlite::Row) -> Result<O>,
}

impl<O> Returning<O> {
    /// Constructs `Returning` from a single column or a tuple of columns.
    pub fn from_columns<C: ReturningColumns<Output = O>>(cols: C) -> Self {
        Self {
            projection: cols.projection(),
            parser: C::parser(),
        }
    }
}

impl<O: FromRow> Returning<O> {
    /// Constructs `Returning` from a target model/DTO struct that implements `FromRow`.
    pub fn from_row() -> Self {
        let cols = O::columns().iter().map(|c| c.column_name).collect::<Vec<_>>().join(", ");
        Self {
            projection: cols,
            parser: |row| O::from_row(&Row::new(row)),
        }
    }
}

/// A trait for types that can be used in a `returning` clause
/// (e.g., a single Column or a tuple of Columns).
pub trait ReturningColumns {
    type Output;

    /// Returns the comma-separated column names for the query string.
    fn projection(&self) -> String;

    /// Returns the parser function to map the database row to the expected output type.
    fn parser() -> fn(&rusqlite::Row) -> Result<Self::Output>;
}

// Support for returning a single column
impl<U: Type> ReturningColumns for Column<U> {
    type Output = U;

    fn projection(&self) -> String {
        self.meta.column_name.to_string()
    }

    fn parser() -> fn(&rusqlite::Row) -> Result<Self::Output> {
        |row| {
            let db_val: Value = row.get(0)
                .map_err(|e| Error::QueryError(e.to_string()))?;
            U::from_db(db_val)
        }
    }
}

macro_rules! impl_returning_columns_for_tuple {
    ( $( $idx:tt => $T:ident ),+ ) => {
        impl<$( $T: Type ),+> ReturningColumns for ($( Column<$T>, )+) {
            type Output = ($( $T, )+);

            fn projection(&self) -> String {
                vec![ $( self.$idx.meta.column_name ),+ ].join(", ")
            }

            fn parser() -> fn(&rusqlite::Row) -> Result<Self::Output> {
                |row| {
                    Ok((
                        $(
                            {
                                let db_val: Value = row.get($idx).map_err(|e| Error::QueryError(e.to_string()))?;
                                $T::from_db(db_val)?
                            }
                        ,)+
                    ))
                }
            }
        }
    };
}

// Support for returning 2 columns: returning((User::id, User::name))
impl_returning_columns_for_tuple!(0 => A, 1 => B);
// Support for returning 3 columns
impl_returning_columns_for_tuple!(0 => A, 1 => B, 2 => C);
// Support for returning 4 columns
impl_returning_columns_for_tuple!(0 => A, 1 => B, 2 => C, 3 => D);