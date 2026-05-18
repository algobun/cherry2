use cherry::{Error, Result, Type, Value};

// Batch implementation for various integer types
macro_rules! impl_db_type_int {
    ($($ty:ty),*) => {
        $(
            impl Type for $ty {
                fn to_db(&self) -> Result<Value> {
                    Ok(Value::Integer(*self as i64))
                }
                fn from_db(value: Value) -> Result<Self> {
                    match value {
                        Value::Integer(i) => Ok(i as $ty),
                        _ => Err(Error::TypeMismatch(format!("Expected Integer for {}", stringify!($ty)))),
                    }
                }
            }
        )*
    };
}

// All integers mapping
impl_db_type_int!(i8, u8, i16, u16, i32, u32, i64, u64, isize, usize);


// For floating-point numbers
macro_rules! impl_db_type_float {
    ($($ty:ty),*) => {
        $(
            impl Type for $ty {
                fn to_db(&self) -> Result<Value> {
                    Ok(Value::Real(*self as f64))
                }
                fn from_db(value: Value) -> Result<Self> {
                    match value {
                        Value::Real(f) => Ok(f as $ty),
                        _ => Err(Error::TypeMismatch(format!("Expected Real for {}", stringify!($ty)))),
                    }
                }
            }
        )*
    };
}

impl_db_type_float!(f32, f64);


// Explicit implementation of String
impl Type for String {
    fn to_db(&self) -> Result<Value> {
        Ok(Value::Text(self.clone()))
    }
    fn from_db(value: Value) -> Result<Self> {
        match value {
            Value::Text(s) => Ok(s),
            _ => Err(Error::TypeMismatch("Expected Text for String".into())),
        }
    }
}


// Explicit implementation of bool
// (SQLite does not have a native boolean, usually using 0 and 1 instead)
impl Type for bool {
    fn to_db(&self) -> Result<Value> {
        Ok(Value::Integer(if *self { 1 } else { 0 }))
    }
    fn from_db(value: Value) -> Result<Self> {
        match value {
            Value::Integer(i) => Ok(i != 0),
            _ => Err(Error::TypeMismatch("Expected Integer for bool".into())),
        }
    }
}


impl Type for Vec<u8> {
    fn to_db(&self) -> Result<Value> {
        // Direct cloning of internal data
        Ok(Value::Blob(self.clone()))
    }

    fn from_db(value: Value) -> Result<Self> {
        match value {
            Value::Blob(b) => Ok(b),
            _ => Err(Error::TypeMismatch("Expected Blob for Vec<u8>".into())),
        }
    }
}


// Automatically implement the mapping of Option<T> for all types T that have implemented Type
impl<T: Type> Type for Option<T> {
    fn to_db(&self) -> Result<Value> {
        match self {
            // If there is a value, call the to_db method of the inner type T
            Some(val) => val.to_db(),
            // If None, directly map to SQLite's NULL
            None => Ok(Value::Null),
        }
    }

    fn from_db(value: Value) -> Result<Self> {
        match value {
            // If NULL is passed from the database, it is safe to parse it as None
            Value::Null => Ok(None),
            // If there is an actual value, pass it to the internal type T for parsing, 
            // then wrap it in Some
            _ => T::from_db(value).map(Some),
        }
    }
}