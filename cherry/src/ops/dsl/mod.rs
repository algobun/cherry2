pub mod column;
pub mod condition;
pub mod assignment;
pub mod order;

#[derive(Copy, Clone, PartialEq)]
pub enum QueryContext {
    Select,
    InsertUpsert, // Trigger `excluded` here
    Update,
    Delete,
}
