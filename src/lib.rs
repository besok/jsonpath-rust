//! # Json path
//! The library provides the basic functionality
//! to find the slice of data according to the query.
//! The idea comes from xpath for xml structures.
//! The details can be found over [`there`]
//! Therefore JSONPath is a query language for JSON,
//! similar to XPath for XML. The jsonpath query is a set of assertions to specify the JSON fields that need to be verified.
//!
//! # Simple example
//! Let's suppose we have a following json:
//! ```json
//!  {
//!   "shop": {
//!    "orders": [
//!       {"id": 1, "active": true},
//!       {"id": 2 },
//!       {"id": 3 },
//!       {"id": 4, "active": true}
//!     ]
//!   }
//! }
//! ```
//! And we pursue to find all orders id having the field 'active'
//! we can construct the jsonpath instance like that
//! ```$.shop.orders[?(@.active)].id``` and get the result ``` [1,4] ```
//!
//! # Another examples
//! ```json
//! { "store": {
//!     "book": [
//!       { "category": "reference",
//!         "author": "Nigel Rees",
//!         "title": "Sayings of the Century",
//!         "price": 8.95
//!       },
//!       { "category": "fiction",
//!         "author": "Evelyn Waugh",
//!         "title": "Sword of Honour",
//!         "price": 12.99
//!       },
//!       { "category": "fiction",
//!         "author": "Herman Melville",
//!         "title": "Moby Dick",
//!         "isbn": "0-553-21311-3",
//!         "price": 8.99
//!       },
//!       { "category": "fiction",
//!         "author": "J. R. R. Tolkien",
//!         "title": "The Lord of the Rings",
//!         "isbn": "0-395-19395-8",
//!         "price": 22.99
//!       }
//!     ],
//!     "bicycle": {
//!       "color": "red",
//!       "price": 19.95
//!     }
//!   }
//! }
//! ```
//! and examples
//! - ``` $.store.book[*].author ``` : the authors of all books in the store
//! - ``` $..book[?(@.isbn)]``` : filter all books with isbn number
//! - ``` $..book[?(@.price<10)]``` : filter all books cheapier than 10
//! - ``` $..*``` : all Elements in XML document. All members of JSON structure
//! - ``` $..book[0,1]``` : The first two books
//! - ``` $..book[:2]``` : The first two books
//!
//! # Operators
//!
//! - `$` : Pointer to the root of the json. It is gently advising to start every jsonpath from the root. Also, inside the filters to point out that the path is starting from the root.
//! - `@`Pointer to the current element inside the filter operations.It is used inside the filter operations to iterate the collection.
//! - `*` or `[*]`Wildcard. It brings to the list all objects and elements regardless their names.It is analogue a flatmap operation.
//! - `<..>`| Descent operation. It brings to the list all objects, children of that objects and etc It is analogue a flatmap operation.
//! - `.<name>` or `.['<name>']`the key pointing to the field of the objectIt is used to obtain the specific field.
//! - `['<name>' (, '<name>')]`the list of keysthe same usage as for a single key but for list
//! - `[<number>]`the filter getting the element by its index.
//! - `[<number> (, <number>)]`the list if elements of array according to their indexes representing these numbers. |
//! - `[<start>:<end>:<step>]`slice operator to get a list of element operating with their indexes. By default step = 1, start = 0, end = array len. The elements can be omitted ```[:]```
//! - `[?(<expression>)]`the logical expression to filter elements in the list.It is used with arrays preliminary.
//!
//! # Examples
//!```rust
//! use std::str::FromStr;
//! use serde_json::{json, Value};
//! use jsonpath_rust::{jp_v, JsonPathValue, JsonPath};
//!
//! fn test() -> Result<(), Box<dyn std::error::Error>> {
//!     let json = serde_json::from_str(r#"{"first":{"second":[{"active":1},{"passive":1}]}}"#)?;
//!     let path = JsonPath::try_from("$.first.second[?(@.active)]")?;
//!     let slice_of_data:Vec<JsonPathValue<Value>> = path.find_slice(&json);
//!     let js = json!({"active":1});
//!     assert_eq!(slice_of_data, jp_v![&js;"$.first.second[0]",]);
//!     # Ok(())
//! }
//! ```
//!
//!
//! [`there`]: https://goessner.net/articles/JsonPath/

pub use parser::model::JsonPath;
pub use parser::JsonPathParserError;
use serde_json::Value;
use std::fmt::Debug;
use std::ops::Deref;
use JsonPathValue::{NewValue, NoValue, Slice};

mod jsonpath;
pub mod parser;
pub mod path;

#[macro_use]
extern crate pest_derive;
extern crate core;
extern crate pest;

/// the trait allows to query a path on any value by just passing the &str of as JsonPath.
///
/// It is equal to
/// ```rust
/// # use serde_json::json;
/// # use std::str::FromStr;
/// use jsonpath_rust::JsonPath;
///
/// let query = "$.hello";
/// let json_path = JsonPath::from_str(query).unwrap();
/// json_path.find(&json!({"hello": "world"}));
/// ```
///
/// It is default implemented for [Value].
///
/// #Note:
/// the result is going to be cloned and therefore it can be significant for the huge queries.
/// if the same &str is used multiple times, it's more efficient to reuse a single JsonPath.
///
/// # Examples:
/// ```
/// use std::str::FromStr;
/// use serde_json::{json, Value};
/// use jsonpath_rust::jp_v;
/// use jsonpath_rust::{JsonPathQuery, JsonPath, JsonPathValue};
///
/// fn test() -> Result<(), Box<dyn std::error::Error>> {
///     let json: Value = serde_json::from_str("{}")?;
///     let v = json.path("$..book[?(@.author size 10)].title")?;
///     assert_eq!(v, json!([]));
///
///     let json: Value = serde_json::from_str("{}")?;
///     let path = json.path("$..book[?(@.author size 10)].title")?;
///
///     assert_eq!(path, json!(["Sayings of the Century"]));
///
///     let json: Value = serde_json::from_str("{}")?;
///     let path = JsonPath::try_from("$..book[?(@.author size 10)].title")?;
///
///     let v = path.find_slice(&json);
///     let js = json!("Sayings of the Century");
///     assert_eq!(v, jp_v![&js;"",]);
///     # Ok(())
/// }
///
/// ```
pub trait JsonPathQuery {
    fn path(self, query: &str) -> Result<Value, JsonPathParserError>;
}

/// Json paths may return either pointers to the original json or new data. This custom pointer type allows us to handle both cases.
/// Unlike JsonPathValue, this type does not represent NoValue to allow the implementation of Deref.
pub enum JsonPtr<'a, Data> {
    /// The slice of the initial json data
    Slice(&'a Data),
    /// The new data that was generated from the input data (like length operator)
    NewValue(Data),
}

/// Allow deref from json pointer to value.
impl<'a> Deref for JsonPtr<'a, Value> {
    type Target = Value;

    fn deref(&self) -> &Self::Target {
        match self {
            JsonPtr::Slice(v) => v,
            JsonPtr::NewValue(v) => v,
        }
    }
}

impl JsonPathQuery for Value {
    fn path(self, query: &str) -> Result<Value, JsonPathParserError> {
        let p = JsonPath::try_from(query)?;
        Ok(p.find(&self))
    }
}

/*
impl<T> JsonPathQuery for T
    where T: Deref<Target=Value> {
    fn path(self, query: &str) -> Result<Value, String> {
        let p = JsonPath::from_str(query)?;
        Ok(p.find(self.deref()))
    }
}
 */

/// just to create a json path value of data
/// Example:
///  - `jp_v(&json) = JsonPathValue::Slice(&json)`
///  - `jp_v(&json;"foo") = JsonPathValue::Slice(&json, "foo".to_string())`
///  - `jp_v(&json,) = vec![JsonPathValue::Slice(&json)]`
///  - `jp_v[&json1,&json1] = vec![JsonPathValue::Slice(&json1),JsonPathValue::Slice(&json2)]`
///  - `jp_v(json) = JsonPathValue::NewValue(json)`
/// ```
/// use std::str::FromStr;
/// use serde_json::{json, Value};
/// use jsonpath_rust::{jp_v, JsonPathQuery, JsonPath, JsonPathValue};
///
/// fn test() -> Result<(), Box<dyn std::error::Error>> {
///     let json: Value = serde_json::from_str("{}")?;
///     let path = JsonPath::try_from("$..book[?(@.author size 10)].title")?;
///     let v = path.find_slice(&json);
///
///     let js = json!("Sayings of the Century");
///     assert_eq!(v, jp_v![&js;"",]);
///     # Ok(())
/// }
/// ```
#[macro_export]
macro_rules! jp_v {
    (&$v:expr) =>{
        JsonPathValue::Slice(&$v, String::new())
    };

    (&$v:expr ; $s:expr) =>{
        JsonPathValue::Slice(&$v, $s.to_string())
    };

    ($(&$v:expr;$s:expr),+ $(,)?) =>{
        {
            vec![
                $(
                    jp_v!(&$v ; $s),
                )+
            ]
        }
    };

    ($(&$v:expr),+ $(,)?) => {
        {
            vec![
                $(
                    jp_v!(&$v),
                )+
            ]
        }
    };

    ($v:expr) =>{
        JsonPathValue::NewValue($v)
    };

}

/// Represents the path of the found json data
type JsPathStr = String;

pub fn jsp_idx(prefix: &str, idx: usize) -> String {
    format!("{}[{}]", prefix, idx)
}
pub fn jsp_obj(prefix: &str, key: &str) -> String {
    format!("{}.['{}']", prefix, key)
}

/// A result of json path
/// Can be either a slice of initial data or a new generated value(like length of array)
#[derive(Debug, PartialEq, Clone)]
pub enum JsonPathValue<'a, Data> {
    /// The slice of the initial json data
    Slice(&'a Data, JsPathStr),
    /// The new data that was generated from the input data (like length operator)
    NewValue(Data),
    /// The absent value that indicates the input data is not matched to the given json path (like the absent fields)
    NoValue,
}

impl<'a, Data: Clone + Debug + Default> JsonPathValue<'a, Data> {
    /// Transforms given value into data either by moving value out or by cloning
    pub fn to_data(self) -> Data {
        match self {
            Slice(r, _) => r.clone(),
            NewValue(val) => val,
            NoValue => Data::default(),
        }
    }

    /// Transforms given value into path
    pub fn to_path(self) -> Option<JsPathStr> {
        match self {
            Slice(_, path) => Some(path),
            _ => None,
        }
    }

    pub fn from_root(data: &'a Data) -> Self {
        Slice(data, String::from("$"))
    }
    pub fn new_slice(data: &'a Data, path: String) -> Self {
        Slice(data, path.to_string())
    }
}

impl<'a, Data> JsonPathValue<'a, Data> {
    pub fn only_no_value(input: &[JsonPathValue<'a, Data>]) -> bool {
        !input.is_empty() && input.iter().filter(|v| v.has_value()).count() == 0
    }

    pub fn map_vec(data: Vec<(&'a Data, JsPathStr)>) -> Vec<JsonPathValue<'a, Data>> {
        data.into_iter()
            .map(|(data, pref)| Slice(data, pref))
            .collect()
    }

    pub fn map_slice<F>(self, mapper: F) -> Vec<JsonPathValue<'a, Data>>
    where
        F: FnOnce(&'a Data, JsPathStr) -> Vec<(&'a Data, JsPathStr)>,
    {
        match self {
            Slice(r, pref) => mapper(r, pref)
                .into_iter()
                .map(|(d, s)| Slice(d, s))
                .collect(),

            NewValue(_) => vec![],
            no_v => vec![no_v],
        }
    }

    pub fn flat_map_slice<F>(self, mapper: F) -> Vec<JsonPathValue<'a, Data>>
    where
        F: FnOnce(&'a Data, JsPathStr) -> Vec<JsonPathValue<'a, Data>>,
    {
        match self {
            Slice(r, pref) => mapper(r, pref),
            _ => vec![NoValue],
        }
    }

    pub fn has_value(&self) -> bool {
        !matches!(self, NoValue)
    }

    pub fn vec_as_data(input: Vec<JsonPathValue<'a, Data>>) -> Vec<&'a Data> {
        input
            .into_iter()
            .filter_map(|v| match v {
                Slice(el, _) => Some(el),
                _ => None,
            })
            .collect()
    }
    pub fn vec_as_pair(input: Vec<JsonPathValue<'a, Data>>) -> Vec<(&'a Data, JsPathStr)> {
        input
            .into_iter()
            .filter_map(|v| match v {
                Slice(el, v) => Some((el, v)),
                _ => None,
            })
            .collect()
    }

    /// moves a pointer (from slice) out or provides a default value when the value was generated
    pub fn slice_or(self, default: &'a Data) -> &'a Data {
        match self {
            Slice(r, _) => r,
            NewValue(_) | NoValue => default,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use crate::JsonPath;
    use std::str::FromStr;

    #[test]
    fn to_string_test() {
        let path: Box<JsonPath<Value>> = Box::from(
            JsonPath::from_str(
                "$.['a'].a..book[1:3][*][1]['a','b'][?(@)][?(@.verb == 'TEST')].a.length()",
            )
            .unwrap(),
        );

        assert_eq!(
            path.to_string(),
            "$.'a'.'a'..book[1:3:1][*][1]['a','b'][?(@ exists )][?(@.'verb' == \"TEST\")].'a'.length()"
        );
    }
}
