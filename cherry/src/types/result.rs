use crate::model::{ColumnMeta, FromRow, Row};
use crate::Result;
use std::borrow::Cow;

/// The FromRow implementation for Option (designed specifically for LEFT JOIN).
impl<T: FromRow> FromRow for Option<T> {
    fn columns() -> Cow<'static, [ColumnMeta]> {
        T::columns()
    }

    fn from_row(row: &Row) -> Result<Self> {
        let cols = T::columns();
        // Detect if the first column of the table is empty
        if let Some(first_col) = cols.first() {
            let alias = format!("{}.{}", first_col.table_name, first_col.column_name);

            // Try to get the underlying rusqlite ValueRef to determine if it is Null or not.
            if let Ok(val_ref) = row.inner.get_ref(alias.as_str()) {
                if val_ref == rusqlite::types::ValueRef::Null {
                    return Ok(None); // Left join did not find related data, return None decisively
                }
            }
        }

        // Data exists, parsing normally T
        T::from_row(row).map(Some)
    }
}


/// The macro-based FromRow implementation for tuples (dynamically concatenating columns).
macro_rules! impl_from_row_for_tuple {
    ( $($T:ident),+ ) => {
        impl<$($T: FromRow),+> FromRow for ($($T,)+) {
            fn columns() -> Cow<'static, [ColumnMeta]> {
                let mut cols = Vec::new();
                $(
                    // Extract each member's column and collect them as Owned
                    cols.extend($T::columns().into_owned());
                )+
                Cow::Owned(cols)
            }

            fn from_row(row: &Row) -> Result<Self> {
                Ok((
                    $( $T::from_row(row)?, )+
                ))
            }
        }
    };
}

// Supports mapping of join table query results from 2 to 26 tables
impl_from_row_for_tuple!(A, B);
impl_from_row_for_tuple!(A, B, C);
impl_from_row_for_tuple!(A, B, C, D);
impl_from_row_for_tuple!(A, B, C, D, E);
impl_from_row_for_tuple!(A, B, C, D, E, F);
impl_from_row_for_tuple!(A, B, C, D, E, F, G);
impl_from_row_for_tuple!(A, B, C, D, E, F, G, H);
impl_from_row_for_tuple!(A, B, C, D, E, F, G, H, I);
impl_from_row_for_tuple!(A, B, C, D, E, F, G, H, I, J);
impl_from_row_for_tuple!(A, B, C, D, E, F, G, H, I, J, K);
impl_from_row_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_from_row_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_from_row_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_from_row_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_from_row_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
impl_from_row_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);
impl_from_row_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R);
impl_from_row_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S);
impl_from_row_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T);
impl_from_row_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U);
impl_from_row_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V);
impl_from_row_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W);
impl_from_row_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X);
impl_from_row_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y);
impl_from_row_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);
