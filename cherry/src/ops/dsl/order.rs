
pub enum OrderDir {
    Asc,
    Desc,
}

pub struct OrderBy {
    pub column: &'static str,
    pub direction: OrderDir,
}