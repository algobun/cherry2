use crate::executor::Executor;
use crate::ops::dsl::condition::Condition;
use crate::ops::dsl::order::{OrderBy, OrderDir};
use crate::ops::dsl::QueryContext;
use crate::query::dsl::Column;
use crate::types::column::{SelectItem, SelectedColumns};
use crate::types::model::{FromRow, Model};
use crate::types::row::Row;
use crate::{Result, Value};
use std::marker::PhantomData;

#[derive(Clone)]
pub(crate) enum JoinType {
    Inner,
    Left,
}

#[derive(Clone)]
pub(crate) struct JoinClause {
    pub(crate) join_type: JoinType,
    pub(crate) table_name: &'static str,
    pub(crate) on_condition: Condition,
}

pub struct Select<'a, T> {
    executor: Executor<'a>,
    ctx: QueryContext,
    is_distinct: bool,
    selected_columns: Vec<SelectItem>,
    joins: Vec<JoinClause>,
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
            joins: Vec::new(),
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

    /// Adds an `INNER JOIN` clause to the query.
    ///
    /// The target table name is automatically deduced from the generic `Model` parameter `M`.
    ///
    /// # Warning: Self-Joins
    /// This method uses the static table name to construct column aliases
    /// (e.g., `"table_name.column_name"`) to prevent column name conflicts between different tables.
    /// As a result, **joining a table to itself (Self-Join) will inevitably cause table and alias collisions**,
    /// leading to SQL syntax errors.
    ///
    /// If you need to perform a self-join or use highly complex alias mappings,
    /// please fall back to the raw SQL interface (`db.raw(...)`).
    pub fn inner_join<M: Model>(mut self, on: Condition) -> Self {
        self.joins.push(JoinClause {
            join_type: JoinType::Inner,
            table_name: M::table_name(),
            on_condition: on,
        });
        self
    }

    /// Adds a `LEFT JOIN` clause to the query.
    ///
    /// The target table name is automatically deduced from the generic `Model` parameter `M`.
    ///
    /// # Warning: Self-Joins
    /// This method uses the static table name to construct column aliases
    /// (e.g., `"table_name.column_name"`) to prevent column name conflicts between different tables.
    /// As a result, **joining a table to itself (Self-Join) will inevitably cause table and alias collisions**,
    /// leading to SQL syntax errors.
    ///
    /// If you need to perform a self-join or use highly complex alias mappings,
    /// please fall back to the raw SQL interface (`db.raw(...)`).
    pub fn left_join<M: Model>(mut self, on: Condition) -> Self {
        self.joins.push(JoinClause {
            join_type: JoinType::Left,
            table_name: M::table_name(),
            on_condition: on,
        });
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
        self.group_bys.push(col.meta.column_name);
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
    pub fn load_as<U: FromRow>(mut self) -> Result<Vec<U>> {
        // Filling Empty Columns
        if self.selected_columns.is_empty() {
            self.selected_columns = U::columns()
                .into_owned()
                .into_iter()
                .map(SelectItem::Column)
                .collect();
        }

        let (sql, params) = build_sql_and_params::<T>(
            self.is_distinct,
            &self.selected_columns,
            &self.joins,
            &self.condition,
            &self.group_bys,
            &self.having,
            &self.order_bys,
            self.limit,
            self.offset,
            self.ctx.clone(),
        )?;

        self.executor.with_conn(|conn| {
            let mut stmt = conn.prepare(&sql)?;
            let mut rows = stmt.query(rusqlite::params_from_iter(params))?;

            let mut result = Vec::new();
            while let Some(rusqlite_row) = rows.next()? {
                result.push(U::from_row(&Row::new(rusqlite_row))?);
            }

            Ok(result)
        })
    }

    /// Quick count query. Overrides `columns`, `order_by`, `limit`, and `offset`.
    pub fn count(mut self) -> Result<usize> {
        self.selected_columns = vec![SelectItem::Raw("count(1)".into())];
        self.order_bys.clear();
        self.limit = None;
        self.offset = None;
        let (sql, params) = build_sql_and_params::<T>(
            self.is_distinct,
            &self.selected_columns,
            &self.joins,
            &self.condition,
            &self.group_bys,
            &self.having,
            &self.order_bys,
            self.limit,
            self.offset,
            self.ctx.clone(),
        )?;

        self.executor.with_conn(|conn| {
            let count: i64 = conn.query_row(
                &sql,
                rusqlite::params_from_iter(params),
                |r| r.get(0)
            )?;

            Ok(count as usize)
        })
    }
}

/// Helper function to compile all collected AST fragments into executable SQL and parameters.
fn build_sql_and_params<T: Model>(
    is_distinct: bool,
    selected_columns: &[SelectItem],
    joins: &[JoinClause],
    condition: &Option<Condition>,
    group_bys: &[&str],
    having: &Option<Condition>,
    order_bys: &[OrderBy],
    limit: Option<usize>,
    offset: Option<usize>,
    ctx: QueryContext,
) -> Result<(String, Vec<Value>)> {
    let mut params = Vec::new();
    let mut sql = String::from("select ");

    if is_distinct {
        sql.push_str("distinct ");
    }

    // Different splicing strategies based on enumerated variants
    let col_part = selected_columns.iter().map(|item| {
        match item {
            // Automatically add "table.column" as a conflict-resistant alias.
            SelectItem::Column(c) => format!("{0}.{1} AS \"{0}.{1}\"", c.table_name, c.column_name),
            SelectItem::Raw(raw_sql) => raw_sql.clone(), // e.g., "count(1)"
        }
    }).collect::<Vec<_>>().join(", ");

    sql.push_str(&col_part);

    sql.push_str(" from ");
    sql.push_str(T::table_name());

    for join in joins {
        match join.join_type {
            JoinType::Inner => sql.push_str(" inner join "),
            JoinType::Left => sql.push_str(" left join "),
        }
        sql.push_str(join.table_name);
        sql.push_str(" on ");
        // Use the Condition AST's build capability directly
        join.on_condition.clone().build(&mut sql, &mut params, ctx.clone())?;
    }

    if let Some(cond) = condition {
        sql.push_str(" where ");
        // Requires Condition to implement Clone
        cond.clone().build(&mut sql, &mut params, ctx.clone())?;
    }

    if !group_bys.is_empty() {
        sql.push_str(" group by ");
        sql.push_str(&group_bys.join(", "));
    }

    if let Some(having_cond) = having {
        sql.push_str(" having ");
        having_cond.clone().build(&mut sql, &mut params, ctx)?;
    }

    if !order_bys.is_empty() {
        sql.push_str(" order by ");
        let orders = order_bys
            .iter()
            .map(|o| {
                format!(
                    "{} {}",
                    o.column.column_name,
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

    Ok((sql, params))
}
