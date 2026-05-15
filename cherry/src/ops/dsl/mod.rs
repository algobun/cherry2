pub mod column;
pub mod condition;
pub mod assignment;
pub mod order;
pub mod returning;

#[derive(Copy, Clone, PartialEq)]
pub enum QueryContext {
    // Insert,
    InsertUpsert, // Trigger `excluded` here
    Update,
    Select,
    Delete,
}
