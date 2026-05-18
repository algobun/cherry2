use crate::types::ColumnMeta;

pub enum OrderDir {
    Asc,
    Desc,
}

pub struct OrderBy {
    pub column: ColumnMeta,
    pub direction: OrderDir,
}