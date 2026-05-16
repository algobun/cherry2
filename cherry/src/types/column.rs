use crate::model::ColumnMeta;
use cherry::query::dsl::Column;


#[derive(Clone)]
pub enum SelectItem {
    /// Representatives a physical table column，For automatic generation of `table.column AS "table.column"`
    Column(ColumnMeta),
    /// Representatives Raw SQL fragment，e.g., "count(1)", "sum(price) AS total"
    Raw(String),
}

impl From<ColumnMeta> for SelectItem {
    fn from(meta: ColumnMeta) -> Self {
        SelectItem::Column(meta)
    }
}


/// A trait to extract column names for the SELECT clause.
/// Strictly limited to `Column<T>` and tuples of `Column<T>` to ensure type safety.
pub(crate) trait SelectedColumns {
    fn to_columns(&self) -> Vec<SelectItem>;
}

impl SelectedColumns for &str {
    fn to_columns(&self) -> Vec<SelectItem> {
        vec![SelectItem::Raw(self.to_string())]
    }
}

// Support for a single Column
impl<T> SelectedColumns for Column<T> {
    fn to_columns(&self) -> Vec<SelectItem> {
        vec![SelectItem::Column(self.meta)]
        // vec![format!("{}.{} AS \"{}.{}\"",
        //      self.meta.table_name, self.meta.column_name,
        //      self.meta.table_name, self.meta.column_name
        // )]
    }
}

// Automatically implement the trait for tuples of mixed Column types
macro_rules! impl_selected_columns_for_tuple {
    ( $($T:ident),+ ) => {
        // `$T` represents any type that implements `SelectedColumns`
        impl<$($T: SelectedColumns),+> SelectedColumns for ($($T,)+) {
            fn to_columns(&self) -> Vec<SelectItem> {
                #[allow(non_snake_case)]
                let ($($T,)+) = self; // Deconstruct the tuple
                let mut cols = Vec::new();
                $( cols.extend($T.to_columns()); )+ // Collect all names
                cols
            }
        }
    };
}

// Support for up to 26 columns
impl_selected_columns_for_tuple!(A, B);
impl_selected_columns_for_tuple!(A, B, C);
impl_selected_columns_for_tuple!(A, B, C, D);
impl_selected_columns_for_tuple!(A, B, C, D, E);
impl_selected_columns_for_tuple!(A, B, C, D, E, F);
impl_selected_columns_for_tuple!(A, B, C, D, E, F, G);
impl_selected_columns_for_tuple!(A, B, C, D, E, F, G, H);
impl_selected_columns_for_tuple!(A, B, C, D, E, F, G, H, I);
impl_selected_columns_for_tuple!(A, B, C, D, E, F, G, H, I, J);
impl_selected_columns_for_tuple!(A, B, C, D, E, F, G, H, I, J, K);
impl_selected_columns_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_selected_columns_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_selected_columns_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_selected_columns_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_selected_columns_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
impl_selected_columns_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);
impl_selected_columns_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R);
impl_selected_columns_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S);
impl_selected_columns_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T);
impl_selected_columns_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U);
impl_selected_columns_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V);
impl_selected_columns_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W);
impl_selected_columns_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X);
impl_selected_columns_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y);
impl_selected_columns_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);