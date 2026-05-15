use crate::executor::Executor;
use crate::ops::dsl::condition::{Condition, QueryContext};
use crate::ops::dsl::order::{OrderBy, OrderDir};
use crate::query::dsl::Column;
use crate::types::model::Model;
use crate::types::row::{FromRow, Row};
use crate::{DbValue, Result};
use std::marker::PhantomData;

pub struct Select<'a, T> {
    executor: Executor<'a>,
    ctx: QueryContext,
    is_distinct: bool,
    selected_columns: Vec<&'static str>,
    condition: Option<Condition>,
    group_bys: Vec<&'static str>,
    having: Option<Condition>,
    order_bys: Vec<OrderBy>,
    limit: Option<usize>,
    offset: Option<usize>,
    _marker: PhantomData<T>,
}

impl<'a, T: Model> Select<'a, T> {
    pub(crate) fn create(executor: Executor<'a>) -> Self {
        Self {
            executor,
            ctx: QueryContext::Select,
            is_distinct: false,
            selected_columns: Vec::new(),
            condition: None,
            group_bys: Vec::new(),
            having: None,
            order_bys: Vec::new(),
            limit: None,
            offset: None,
            _marker: PhantomData,
        }
    }

    pub fn distinct(mut self) -> Self {
        self.is_distinct = true;
        self
    }

    /// Restricts queries to specific columns.
    /// Supports a single Column, a tuple of mixed Columns, or raw static strings.
    pub fn columns<C: SelectedColumns>(mut self, cols: C) -> Self {
        self.selected_columns = cols.to_columns();
        self
    }

    /// Adds a `where` condition to the select statement.
    /// Calling this multiple times will automatically combine them with `AND`.
    pub fn filter(mut self, cond: Condition) -> Self {
        self.condition = match self.condition {
            Some(existing) => Some(existing & cond),
            None => Some(cond),
        };
        self
    }

    /// Adds a `group by` clause.
    pub fn group_by<C>(mut self, col: Column<C>) -> Self {
        self.group_bys.push(col.name);
        self
    }

    /// Sets the `having` condition for grouped results.
    pub fn having(mut self, cond: Condition) -> Self {
        self.having = Some(cond);
        self
    }

    /// Adds an `order by` clause. Can be chained for multiple sort criteria.
    pub fn order_by(mut self, order: OrderBy) -> Self {
        self.order_bys.push(order);
        self
    }

    /// Sets the maximum number of records to return.
    pub fn limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }

    /// Sets the number of records to skip before starting to return results.
    pub fn offset(mut self, n: usize) -> Self {
        self.offset = Some(n);
        self
    }

    /// Default load returning a `Vec` of the full Model `T`.
    pub fn load(self) -> Result<Vec<T>> {
        self.load_as::<T>()
    }

    /// Advanced Projection Query: Load as any structure `U` that implements `FromRow`.
    pub fn load_as<U: FromRow>(self) -> Result<Vec<U>> {
        let (sql, params) = build_sql_and_params::<T>(
            self.is_distinct,
            &self.selected_columns,
            &self.condition,
            &self.group_bys,
            &self.having,
            &self.order_bys,
            self.limit,
            self.offset,
            self.ctx.clone(),
        );

        let conn = self.executor.conn();
        let mut stmt = conn.prepare(&sql)?;
        let mut rows = stmt.query(rusqlite::params_from_iter(params))?;

        let mut result = Vec::new();
        while let Some(rusqlite_row) = rows.next()? {
            result.push(U::from_row(&Row::new(rusqlite_row))?);
        }

        Ok(result)
    }

    /// Quick count query. Overrides `columns`, `order_by`, `limit`, and `offset`.
    pub fn count(mut self) -> Result<usize> {
        self.selected_columns = vec!["count(1)"];
        self.order_bys.clear();
        self.limit = None;
        self.offset = None;
        let (sql, params) = build_sql_and_params::<T>(
            self.is_distinct,
            &self.selected_columns,
            &self.condition,
            &self.group_bys,
            &self.having,
            &self.order_bys,
            self.limit,
            self.offset,
            self.ctx.clone(),
        );

        let conn = self.executor.conn();
        let count: i64 = conn.query_row(
            &sql,
            rusqlite::params_from_iter(params),
            |r| r.get(0)
        )?;

        Ok(count as usize)
    }
}

/// Helper function to compile all collected AST fragments into executable SQL and parameters.
fn build_sql_and_params<T: Model>(
    is_distinct: bool,
    selected_columns: &[&str],
    condition: &Option<Condition>,
    group_bys: &[&str],
    having: &Option<Condition>,
    order_bys: &[OrderBy],
    limit: Option<usize>,
    offset: Option<usize>,
    ctx: QueryContext,
) -> (String, Vec<DbValue>) {
    let mut params = Vec::new();
    let mut sql = String::from("select ");

    if is_distinct {
        sql.push_str("distinct ");
    }

    let col_part = if selected_columns.is_empty() {
        T::columns().iter().map(|c| c.name).collect::<Vec<_>>().join(", ")
    } else {
        selected_columns.join(", ")
    };
    sql.push_str(&col_part);

    sql.push_str(" from ");
    sql.push_str(T::table_name());

    if let Some(cond) = condition {
        sql.push_str(" where ");
        // Requires Condition to implement Clone
        cond.clone().build(&mut sql, &mut params, ctx.clone());
    }

    if !group_bys.is_empty() {
        sql.push_str(" group by ");
        sql.push_str(&group_bys.join(", "));
    }

    if let Some(having_cond) = having {
        sql.push_str(" having ");
        having_cond.clone().build(&mut sql, &mut params, ctx);
    }

    if !order_bys.is_empty() {
        sql.push_str(" order by ");
        let orders = order_bys
            .iter()
            .map(|o| {
                format!(
                    "{} {}",
                    o.column,
                    match o.direction {
                        OrderDir::Asc => "asc",
                        OrderDir::Desc => "desc",
                    }
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        sql.push_str(&orders);
    }

    if let Some(l) = limit {
        sql.push_str(&format!(" limit {}", l));
    }
    if let Some(o) = offset {
        sql.push_str(&format!(" offset {}", o));
    }

    (sql, params)
}


/// A trait to extract column names for the SELECT clause.
/// Strictly limited to `Column<T>` and tuples of `Column<T>` to ensure type safety.
pub(crate) trait SelectedColumns {
    fn to_columns(&self) -> Vec<&'static str>;
}

// Support for a single Column
impl<T> SelectedColumns for Column<T> {
    fn to_columns(&self) -> Vec<&'static str> {
        vec![self.name]
    }
}

// Automatically implement the trait for tuples of mixed Column types
macro_rules! impl_selected_columns_for_tuple {
    ( $($T:ident),+ ) => {
        // `$T` represents any type that implements `SelectedColumns`
        impl<$($T: SelectedColumns),+> SelectedColumns for ($($T,)+) {
            fn to_columns(&self) -> Vec<&'static str> {
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