use crate::core::value::{Type, Value};
use crate::{Error, Result};
use std::convert::TryFrom;
use std::num::*;
use std::sync::Arc;

// Batch implementation for various integer types
macro_rules! impl_db_type_int {
    ($($ty:ty),*) => {
        $(
            impl Type for $ty {
                fn to_db(&self) -> Result<Value> {
                    // Safe conversion: reject oversized u64 into negative numbers
                    i64::try_from(*self)
                        .map(Value::Integer)
                        .map_err(|_| Error::ConversionError(format!("Integer overflow converting {} to i64", stringify!($ty))))
                }
                fn from_db(value: Value) -> Result<Self> {
                    match value {
                        Value::Integer(i) => {
                            // Safe parsing: refuse to force large values into small containers (e.g. i64 into i8).
                            <$ty>::try_from(i)
                                .map_err(|_| Error::ParseError(format!("Integer out of bounds for {}", stringify!($ty))))
                        },
                        _ => Err(Error::TypeMismatch(format!("Expected Integer for {}", stringify!($ty)))),
                    }
                }
            }
        )*
    };
}

// All integers mapping
impl_db_type_int!(i8, u8, i16, u16, i32, u32, i64, u64, isize, usize);

macro_rules! impl_db_type_nonzero {
    ($($nz_ty:ident, $base_ty:ty),*) => {
        $(
            impl Type for $nz_ty {
                fn to_db(&self) -> Result<Value> {
                    // Get the underlying primitive types directly, reusing the to_db written for the base types.
                    self.get().to_db()
                }
                fn from_db(value: Value) -> Result<Self> {
                    // Parsed into base types first
                    let base_val = <$base_ty>::from_db(value)?;
                    // Trying to convert to NonZero
                    $nz_ty::new(base_val)
                        .ok_or_else(|| Error::ParseError(format!("Zero is invalid for {}", stringify!($nz_ty))))
                }
            }
        )*
    };
}

impl_db_type_nonzero!(
    NonZeroI8, i8, NonZeroU8, u8,
    NonZeroI16, i16, NonZeroU16, u16,
    NonZeroI32, i32, NonZeroU32, u32,
    NonZeroI64, i64, NonZeroU64, u64,
    NonZeroIsize, isize, NonZeroUsize, usize
);


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


impl Type for &str {
    fn to_db(&self) -> Result<Value> {
        Ok(Value::Text(self.to_string()))
    }
    fn from_db(_value: Value) -> Result<Self> {
        Err(Error::ParseError("Cannot deserialize database value into a borrowed &str. Use String instead.".into()))
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

impl Type for &[u8] {
    fn to_db(&self) -> Result<Value> {
        Ok(Value::Blob(self.to_vec()))
    }
    fn from_db(_value: Value) -> Result<Self> {
        Err(Error::ParseError("Cannot deserialize database value into borrowed &[u8]. Use Vec<u8> instead.".into()))
    }
}

// Support for fixed-length arrays, such as [u8; 16], a common hash/UUID underlying store
impl<const N: usize> Type for [u8; N] {
    fn to_db(&self) -> Result<Value> {
        Ok(Value::Blob(self.to_vec()))
    }
    fn from_db(value: Value) -> Result<Self> {
        match value {
            Value::Blob(b) => {
                b.try_into()
                    .map_err(|_| Error::ParseError(format!("Expected Blob of length {}", N)))
            }
            _ => Err(Error::TypeMismatch("Expected Blob for fixed-size array".into())),
        }
    }
}



// As long as the internal T implements Type, Box<T> automatically gains the ability
impl<T: Type> Type for Box<T> {
    fn to_db(&self) -> Result<Value> {
        self.as_ref().to_db() // Delegate to internal T
    }
    fn from_db(value: Value) -> Result<Self> {
        T::from_db(value).map(Box::new) // Reboxing after parsing
    }
}

// Concurrency-safe Arc < T >
impl<T: Type> Type for Arc<T> {
    fn to_db(&self) -> Result<Value> {
        self.as_ref().to_db()
    }
    fn from_db(value: Value) -> Result<Self> {
        T::from_db(value).map(Arc::new)
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