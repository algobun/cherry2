use crate::executor::Executor;
use crate::ops::dsl::condition::Condition;
use crate::ops::dsl::order::{OrderBy, OrderDir};
use crate::query::dsl::Column;
use crate::types::model::Model;
use crate::types::row::{FromRow, Row};
use crate::{DbValue, Result};
use std::marker::PhantomData;

pub struct SelectBuilder<'a, T> {
    executor: Executor<'a>,
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

impl<'a, T: Model> SelectBuilder<'a, T> {

    pub(crate) fn create(executor: Executor<'a>) -> Self {
        Self {
            executor,
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

    /// Restrict queries to specific columns
    pub fn columns<C>(mut self, cols: &[Column<C>]) -> Self {
        self.selected_columns = cols.iter().map(|c| c.name).collect();
        self
    }

    /// Filter condition: idempotent operation, direct override
    pub fn filter(mut self, cond: Condition) -> Self {
        self.condition = Some(cond);
        self
    }

    pub fn group_by<C>(mut self, col: Column<C>) -> Self {
        self.group_bys.push(col.name);
        self
    }

    /// Having condition: idempotent operation
    pub fn having(mut self, cond: Condition) -> Self {
        self.having = Some(cond);
        self
    }

    pub fn order_by(mut self, order: OrderBy) -> Self {
        self.order_bys.push(order);
        self
    }

    pub fn limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }

    pub fn offset(mut self, n: usize) -> Self {
        self.offset = Some(n);
        self
    }


    /// Compile all collected AST fragments into executable SQL and parameter slices
    fn build_sql_and_params(&self) -> (String, Vec<DbValue>) {
        let mut params = Vec::new();
        let mut sql = String::from("select ");

        if self.is_distinct { sql.push_str("distinct "); }

        let col_part = if self.selected_columns.is_empty() {
            T::columns().iter().map(|c| c.name).collect::<Vec<_>>().join(", ")
        } else {
            self.selected_columns.join(", ")
        };
        sql.push_str(&col_part);

        sql.push_str(" from ");
        sql.push_str(T::table_name());

        if let Some(cond) = &self.condition {
            sql.push_str(" where ");
            cond.clone().build(&mut sql, &mut params); // 需要 Condition 派生 Clone
        }

        if !self.group_bys.is_empty() {
            sql.push_str(" group by ");
            sql.push_str(&self.group_bys.join(", "));
        }

        if let Some(having) = &self.having {
            sql.push_str(" having ");
            having.clone().build(&mut sql, &mut params);
        }

        if !self.order_bys.is_empty() {
            sql.push_str(" order by ");
            let orders = self.order_bys.iter()
                .map(|o| format!("{} {}", o.column, match o.direction {
                    OrderDir::Asc => "asc",
                    OrderDir::Desc => "desc",
                }))
                .collect::<Vec<_>>().join(", ");
            sql.push_str(&orders);
        }

        if let Some(l) = self.limit {
            sql.push_str(&format!(" limit {}", l));
        }
        if let Some(o) = self.offset {
            sql.push_str(&format!(" offset {}", o));
        }

        (sql, params)
    }


    /// Default loaded as generalized `T` (full Model)
    pub fn load(self) -> Result<Vec<T>> {
        self.load_as::<T>()
    }

    /// Advanced Projection Query: Load as any structure `U` that implements FromRow
    pub fn load_as<U: FromRow>(self) -> Result<Vec<U>> {
        let (sql, params) = self.build_sql_and_params();

        // Use `Executor` to get the connection uniformly
        let conn = self.executor.conn();
        let mut stmt = conn.prepare(&sql)?;

        // Instead of query_map, manually iterate through the query using a more primitive query,
        // which is perfect for unraveling crate::Result on the outside of the closure
        // without being abducted by rusqlite::Error signatures
        let mut rows = stmt.query(rusqlite::params_from_iter(params))?;

        let mut result = Vec::new();
        while let Some(rusqlite_row) = rows.next()? {
            // Wrap it into a Row for yourself, and pass it to the user's from_row implementation
            let my_row = Row::new(rusqlite_row);
            let item = U::from_row(&my_row)?;
            result.push(item);
        }

        Ok(result)
    }

    /// Quick count query
    pub fn count(mut self) -> Result<usize> {
        self.selected_columns = vec!["count(1)"];
        self.order_bys.clear();
        self.limit = None;
        self.offset = None;

        let (sql, params) = self.build_sql_and_params();
        let conn = self.executor.conn();

        // `query_row` also supports automatic return of a single simple type
        let count: i64 = conn.query_row(
            &sql,
            rusqlite::params_from_iter(params), |r| r.get(0)
        )?;
        Ok(count as usize)
    }
}
