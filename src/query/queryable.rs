use crate::query::state::Data;
use serde_json::{json, Value};
use std::fmt::Debug;

pub trait Queryable
where
    Self: Default
        + Clone
        + Debug
        + for<'a> From<&'a str>
        + From<bool>
        + From<i64>
        + From<f64>
        + From<Vec<Self>>
        + From<String>
        + PartialEq
{


    /// Retrieves a reference to the value associated with the given key.
    fn get(&self, key: &str) -> Option<&Self>;

    fn as_array(&self) -> Option<&Vec<Self>>;

    fn as_object(&self) -> Option<Vec<(&String, &Self)>>;

    fn as_str(&self) -> Option<&str>;

    fn as_i64(&self) -> Option<i64>;

    /// Returns a null value.
    fn null() -> Self;


}

impl Queryable for Value {

    fn get(&self, key: &str) -> Option<&Self> {
        self.get(key)
    }

    fn as_array(&self) -> Option<&Vec<Self>> {
        self.as_array()
    }

    fn as_object(&self) -> Option<Vec<(&String, &Self)>> {
        self.as_object()
            .map(|v| v.into_iter().map(|(k, v)| (k, v)).collect())
    }

    fn as_str(&self) -> Option<&str> {
        self.as_str()
    }

    fn as_i64(&self) -> Option<i64> {
        self.as_i64()
    }

    fn null() -> Self {
        Value::Null
    }
}
