use crate::query::state::Data;
use crate::{JsonPathParserError, JsonPathStr};
use serde_json::{json, Value};
use std::borrow::Cow;
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
        + PartialEq,
{
    /// Retrieves a reference to the value associated with the given key.
    /// It is the responsibility of the implementation to handle enclosing single and double quotes.
    fn get(&self, key: &str) -> Option<&Self>;

    fn as_array(&self) -> Option<&Vec<Self>>;

    fn as_object(&self) -> Option<Vec<(&String, &Self)>>;

    fn as_str(&self) -> Option<&str>;

    fn as_i64(&self) -> Option<i64>;
    fn as_bool(&self) -> Option<bool>;

    /// Returns a null value.
    fn null() -> Self;

    fn extension_custom(_name: &str, _args: Vec<Cow<Self>>) -> Self {
        Self::null()
    }
}

impl Queryable for Value {
    fn get(&self, key: &str) -> Option<&Self> {
        let key = key.trim_matches(|c| c == '\'' || c == '"').trim();
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

    fn as_bool(&self) -> Option<bool> {
        self.as_bool()
    }

    fn null() -> Self {
        Value::Null
    }
}
