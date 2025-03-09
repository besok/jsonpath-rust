use crate::query::state::Data;
use crate::{JsonPathParserError, JsonPathStr};
use serde_json::{json, Value};
use std::borrow::Cow;
use std::fmt::Debug;
use crate::query::QueryPath;

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
    /// The key will be normalized (quotes trimmed, whitespace handled, the escape symbols handled) before lookup.
    fn get(&self, key: &str) -> Option<&Self>;

    fn as_array(&self) -> Option<&Vec<Self>>;

    fn as_object(&self) -> Option<Vec<(&String, &Self)>>;

    fn as_str(&self) -> Option<&str>;

    fn as_i64(&self) -> Option<i64>;
    fn as_f64(&self) -> Option<f64>;
    fn as_bool(&self) -> Option<bool>;

    /// Returns a null value.
    fn null() -> Self;

    fn extension_custom(_name: &str, _args: Vec<Cow<Self>>) -> Self {
        Self::null()
    }

    /// Retrieves a reference to the element at the specified path.
    /// The path is specified as a string and can be obtained from the query.
    ///
    /// # Arguments
    /// * `path` -  A json path to the element specified as a string (root, field, index only).
    fn reference<T>(&self, path:T) -> Option<&Self> where T:Into<QueryPath> {
        None
    }

    /// Retrieves a mutable reference to the element at the specified path.
    ///
    /// # Arguments
    /// * `path` -  A json path to the element specified as a string (root, field, index only).
    ///
    /// # Examples
    ///
    /// ```
    /// use serde_json::json;
    /// use jsonpath_rust::{JsonPath, JsonPathParserError};
    /// use jsonpath_rust::path::JsonLike;
    ///
    /// let mut json = json!([
    ///     {"verb": "RUN","distance":[1]},
    ///     {"verb": "TEST"},
    ///     {"verb": "DO NOT RUN"}
    /// ]);
    ///
    /// let path: Box<JsonPath> = Box::from(JsonPath::try_from("$.[?@.verb == 'RUN']").unwrap());
    /// let elem = path
    ///     .find_as_path(&json)
    ///     .get(0)
    ///     .cloned()
    ///     .ok_or(JsonPathParserError::InvalidJsonPath("".to_string())).unwrap();
    ///
    /// if let Some(v) = json
    ///     .reference_mut(elem).unwrap()
    ///     .and_then(|v| v.as_object_mut())
    ///     .and_then(|v| v.get_mut("distance"))
    ///     .and_then(|v| v.as_array_mut())
    /// {
    ///     v.push(json!(2))
    /// }
    ///
    /// assert_eq!(
    ///     json,
    ///     json!([
    ///         {"verb": "RUN","distance":[1,2]},
    ///         {"verb": "TEST"},
    ///         {"verb": "DO NOT RUN"}
    ///     ])
    /// );
    /// ```
    fn reference_mut<T>(&mut self, path:T) -> Option<&Self> where T:Into<QueryPath> {
        None
    }
}

impl Queryable for Value {
    fn get(&self, key: &str) -> Option<&Self> {
        let key = if key.starts_with("'") && key.ends_with("'") {
            key.trim_matches(|c| c == '\'')
        } else if key.starts_with('"') && key.ends_with('"') {
            key.trim_matches(|c| c == '"')
        } else {
            key
        };

        self.get(key.trim())
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

    fn as_f64(&self) -> Option<f64> {
        self.as_f64()
    }

    fn as_bool(&self) -> Option<bool> {
        self.as_bool()
    }

    fn null() -> Self {
        Value::Null
    }

    fn reference<T>(&self, path: T) -> Option<&Self>
    where
        T: Into<QueryPath>,
    {
        todo!()
    }
}
