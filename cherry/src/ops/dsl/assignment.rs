use crate::types::value::DbValue;

pub enum Assignment {
    /// Corresponding `update t set col = ?`
    Value(&'static str, DbValue),
    /// Corresponding `update t set col = col + ?`
    Increment(&'static str, DbValue),
    /// Corresponding `do update set col = excluded.col` (No placeholder required.)
    Excluded(&'static str),
    /// Corresponding `do update t set col = excluded.col + ?`
    ExcludedIncrement(&'static str, DbValue),
}


pub fn build_assignments(assignments: Vec<Assignment>, sql: &mut String, params: &mut Vec<DbValue>) {
    let mut is_first = true;
    for assign in assignments {
        if !is_first {
            sql.push_str(", ");
        }

        match assign {
            Assignment::Value(col, val) => {
                // 普通设值: col = ?
                sql.push_str(col);
                sql.push_str(" = ?");
                params.push(val);
            }
            Assignment::Increment(col, val) => {
                sql.push_str(col);
                sql.push_str(" = ");
                sql.push_str(col);
                sql.push_str(" + ?");
                params.push(val);
            },
            Assignment::Excluded(col) => {
                // 冲突覆盖: col = excluded.col
                sql.push_str(col);
                sql.push_str(" = excluded.");
                sql.push_str(col);
                // No params.push()
            }
            Assignment::ExcludedIncrement(col, val) => {
                sql.push_str(col);
                sql.push_str(" = excluded.");
                sql.push_str(col);
                sql.push_str(" + ?");
                params.push(val);
            }

        }
        is_first = false;
    }
}