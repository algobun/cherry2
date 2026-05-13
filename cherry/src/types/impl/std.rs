use cherry::{DbType, DbValue, Result, Error};

// Batch implementation for various integer types
macro_rules! impl_db_type_int {
    ($($ty:ty),*) => {
        $(
            impl DbType for $ty {
                fn to_db_value(&self) -> DbValue {
                    DbValue::Integer(*self as i64)
                }
                fn from_db_value(value: DbValue) -> Result<Self> {
                    match value {
                        DbValue::Integer(i) => Ok(i as $ty),
                        _ => Err(Error::TypeMismatch(format!("Expected Integer for {}", stringify!($ty)))),
                    }
                }
            }
        )*
    };
}

// All integers mapping
impl_db_type_int!(i8, u8, i16, u16, i32, u32, i64);


// For floating-point numbers
macro_rules! impl_db_type_float {
    ($($ty:ty),*) => {
        $(
            impl DbType for $ty {
                fn to_db_value(&self) -> DbValue {
                    DbValue::Real(*self as f64)
                }
                fn from_db_value(value: DbValue) -> Result<Self> {
                    match value {
                        DbValue::Real(f) => Ok(f as $ty),
                        _ => Err(Error::TypeMismatch(format!("Expected Real for {}", stringify!($ty)))),
                    }
                }
            }
        )*
    };
}

impl_db_type_float!(f32, f64);


// Explicit implementation of String
impl DbType for String {
    fn to_db_value(&self) -> DbValue {
        DbValue::Text(self.clone())
    }
    fn from_db_value(value: DbValue) -> cherry::Result<Self> {
        match value {
            DbValue::Text(s) => Ok(s),
            _ => Err(Error::TypeMismatch("Expected Text for String".into())),
        }
    }
}


// Explicit implementation of bool
// (SQLite does not have a native boolean, usually using 0 and 1 instead)
impl DbType for bool {
    fn to_db_value(&self) -> DbValue {
        DbValue::Integer(if *self { 1 } else { 0 })
    }
    fn from_db_value(value: DbValue) -> cherry::Result<Self> {
        match value {
            DbValue::Integer(i) => Ok(i != 0),
            _ => Err(Error::TypeMismatch("Expected Integer for bool".into())),
        }
    }
}


// Automatically implement the mapping of Option<T> for all types T that have implemented DbType
impl<T: DbType> DbType for Option<T> {
    fn to_db_value(&self) -> DbValue {
        match self {
            // If there is a value, call the to_db_value method of the inner type T
            Some(val) => val.to_db_value(),
            // If None, directly map to SQLite's NULL
            None => DbValue::Null,
        }
    }

    fn from_db_value(value: DbValue) -> Result<Self> {
        match value {
            // If NULL is passed from the database, it is safe to parse it as None
            DbValue::Null => Ok(None),
            // If there is an actual value, pass it to the internal type T for parsing, 
            // then wrap it in Some
            _ => T::from_db_value(value).map(Some),
        }
    }
}