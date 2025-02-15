use crate::query::Step;
use serde_json::{json, Value};
use std::fmt::Debug;

pub trait Queryable
where
    Self: Default
        + Clone
        + Debug
        + for<'a> From<&'a str>
        + From<Vec<String>>
        + From<bool>
        + From<i64>
        + From<f64>
        + From<Vec<Self>>
        + From<String>
        + PartialEq,
{
    /// Returns the length/size of the object.
    ///
    /// # Returns
    ///
    /// Returns a `Progress` enum containing either:
    /// - `Progress::Data` with a vector of references to self and the query path for strings/arrays/objects
    /// - `Progress::Nothing` for other types
    ///
    /// The returned length follows JSON path length() function semantics based on the type:
    /// - String type: Number of Unicode scalar values
    /// - Array type: Number of elements
    /// - Object type: Number of members
    /// - Other types: Nothing
    fn extension_length<'a>(&self) -> Step<'a, Self>;

    fn extension_custom<'a>(&self, _name: &str) -> Step<'a, Self> {
        Step::Nothing
    }

    /// Retrieves a reference to the value associated with the given key.
    fn get(&self, key: &str) -> Option<&Self>;

    /// Returns true if the value is an  array.
    fn is_array(&self) -> bool;

    /// If the value is an array, returns a reference to the array.
    /// Returns None if the value is not an array.
    fn as_array(&self) -> Option<&Vec<Self>>;
}

impl Queryable for Value {
    fn extension_length<'a>(&self) -> Step<'a, Self> {
        match self {
            Value::String(s) => Step::NewData(json!(s.chars().count())),
            Value::Array(elems) => Step::NewData(json!(elems.len())),
            Value::Object(elems) => Step::NewData(json!(elems.len())),
            _ => Step::Nothing,
        }
    }

    fn get(&self, key: &str) -> Option<&Self> {
        self.get(key)
    }

    fn is_array(&self) -> bool {
        self.is_array()
    }

    fn as_array(&self) -> Option<&Vec<Self>> {
        self.as_array()
    }
}
