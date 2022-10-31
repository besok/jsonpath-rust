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
//!```
//! use serde_json::{json,Value};
//! use jsonpath_rust::json_path_value;
//! use self::jsonpath_rust::JsonPathFinder;
//! use self::jsonpath_rust::JsonPathValue;
//! fn test(){
//!     let  finder = JsonPathFinder::from_str(r#"{"first":{"second":[{"active":1},{"passive":1}]}}"#, "$.first.second[?(@.active)]").unwrap();
//!     let slice_of_data:Vec<JsonPathValue<Value>> = finder.find_slice();
//!     let js = json!({"active":1});
//!     assert_eq!(slice_of_data, json_path_value![&js,]);
//! }
//! ```
//! or even simpler:
//!
//!```
//! use serde_json::{json,Value};
//! use self::jsonpath_rust::JsonPathFinder;
//! use self::jsonpath_rust::JsonPathValue;
//! fn test(json: &str, path: &str, expected: Vec<JsonPathValue<Value>>) {
//!    match JsonPathFinder::from_str(json, path) {
//!        Ok(finder) => assert_eq!(finder.find_slice(), expected),
//!        Err(e) => panic!("error while parsing json or jsonpath: {}", e)
//!    }
//!
//!
//! }
//! ```
//!
//! [`there`]: https://goessner.net/articles/JsonPath/


use std::fmt::Debug;
use std::str::FromStr;
use serde_json::{Value};
use crate::parser::parser::parse_json_path;
use crate::path::{json_path_instance, PathInstance};
use crate::parser::model::JsonPath;

mod parser;
mod path;


#[macro_use]
extern crate pest_derive;
extern crate pest;
extern crate core;

/// the trait allows to mix the method path to the value of [Value]
/// and thus the using can be shortened to the following one:
/// # Examples:
/// ```
/// use std::str::FromStr;
/// use serde_json::{json,Value};
/// use jsonpath_rust::json_path_value;
/// use crate::jsonpath_rust::{JsonPathFinder,JsonPathQuery,JsonPathInst,JsonPathValue};
///fn test(){
///         let json: Value = serde_json::from_str("{}").unwrap();
///         let v = json.path("$..book[?(@.author size 10)].title").unwrap();
///         assert_eq!(v, json!([]));
///
///         let json: Value = serde_json::from_str("{}").unwrap();
///         let path = json.path("$..book[?(@.author size 10)].title").unwrap();
///
///         assert_eq!(path, json!(["Sayings of the Century"]));
///
///         let json: Box<Value> = serde_json::from_str("{}").unwrap();
///         let path: Box<JsonPathInst> = Box::from(JsonPathInst::from_str("$..book[?(@.author size 10)].title").unwrap());
///         let  finder = JsonPathFinder::new(json, path);
///
///         let v = finder.find_slice();
///         let js = json!("Sayings of the Century");
///         assert_eq!(v, json_path_value![&js,]);
///     }
///
/// ```
/// #Note:
/// the result is going to be cloned and therefore it can be significant for the huge queries
pub trait JsonPathQuery {
    fn path(self, query: &str) -> Result<Value, String>;
}

pub struct JsonPathInst {
    inner: JsonPath,
}


impl FromStr for JsonPathInst {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        JsonPath::from_str(s).map(|inner| JsonPathInst { inner })
    }
}

impl JsonPathQuery for Box<Value> {
    fn path(self, query: &str) -> Result<Value, String> {
        let p = JsonPathInst::from_str(query)?;
        Ok(JsonPathFinder::new(self, Box::new(p)).find())
    }
}

impl JsonPathQuery for Value {
    fn path(self, query: &str) -> Result<Value, String> {
        let p = JsonPathInst::from_str(query)?;
        Ok(JsonPathFinder::new(Box::new(self), Box::new(p)).find())
    }
}


/// just to create a json path value of data
/// Example:
///  - json_path_value(&json) = `JsonPathValue::Slice(&json)`
///  - json_path_value(&json,) = `vec![JsonPathValue::Slice(&json)]`
///  - `json_path_value[&json1,&json1]` = `vec![JsonPathValue::Slice(&json1),JsonPathValue::Slice(&json2)]`
///  - json_path_value(json) = `JsonPathValue::NewValue(json)`
/// ```
/// use std::str::FromStr;
/// use serde_json::{json,Value};
/// use jsonpath_rust::json_path_value;
/// use crate::jsonpath_rust::{JsonPathFinder,JsonPathQuery,JsonPathInst,JsonPathValue};
///fn test(){
///         let json: Box<Value> = serde_json::from_str("{}").unwrap();
///         let path: Box<JsonPathInst> = Box::from(JsonPathInst::from_str("$..book[?(@.author size 10)].title").unwrap());
///         let  finder = JsonPathFinder::new(json, path);
///
///         let v = finder.find_slice();
///         let js = json!("Sayings of the Century");
///         assert_eq!(v, json_path_value![&js,]);
///     }
/// ```
#[macro_export]
macro_rules! json_path_value {
    (&$v:expr) =>{
        JsonPathValue::Slice(&$v)
    };

    ($(&$v:expr),+ $(,)?) =>{
        {
        let mut res = Vec::new();
        $(
           res.push(json_path_value!(&$v));
        )+
        res
        }
    };
    ($v:expr) =>{
        JsonPathValue::NewValue($v)
    };

}


/// A result of json path
/// Can be either a slice of initial data or a new generated value(like length of array)
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum JsonPathValue<'a, Data> {
    Slice(&'a Data),
    NewValue(Data),
}

impl<'a, Data: Clone + Debug> JsonPathValue<'a, Data> {
    /// Transforms given value into data either by moving value out or by cloning
    pub fn to_data(self) -> Data {
        match self {
            JsonPathValue::Slice(r) => r.clone(),
            JsonPathValue::NewValue(val) => val
        }
    }


}

impl<'a, Data> From<&'a Data> for JsonPathValue<'a, Data> {
    fn from(data: &'a Data) -> Self {
        JsonPathValue::Slice(data)
    }
}

impl<'a, Data> JsonPathValue<'a, Data> {
    fn map_slice<F>(self, mapper: F) -> Vec<JsonPathValue<'a, Data>>
        where F: FnOnce(&'a Data) -> Vec<&'a Data>
    {
        match self {
            JsonPathValue::Slice(r) => {
                mapper(r)
                    .into_iter()
                    .map(JsonPathValue::Slice)
                    .collect()
            }
            JsonPathValue::NewValue(_) => vec![]
        }
    }

    pub fn slice_into_vec(input: Vec<JsonPathValue<'a, Data>>) -> Vec<&'a Data> {
        input
            .into_iter()
            .filter_map(|v| match v {
                JsonPathValue::Slice(el) => Some(el),
                _ => None

            })
            .collect()
    }

    /// moves a pointer (from slice) out or provides a default value when the value was generated
    pub fn slice_or(self, default: &'a Data) -> &'a Data {
        match self {
            JsonPathValue::Slice(r) => r,
            JsonPathValue::NewValue(_) => default
        }
    }
}

/// The base structure stitching the json instance and jsonpath instance
pub struct JsonPathFinder {
    json: Box<Value>,
    path: Box<JsonPathInst>,
}

impl JsonPathFinder {
    /// creates a new instance of [JsonPathFinder]
    pub fn new(json: Box<Value>, path: Box<JsonPathInst>) -> Self {
        JsonPathFinder { json, path }
    }

    /// updates a path with a new one
    pub fn set_path(&mut self, path: Box<JsonPathInst>) {
        self.path = path
    }
    /// updates a json with a new one
    pub fn set_json(&mut self, json: Box<Value>) {
        self.json = json
    }
    /// updates a json from string and therefore can be some parsing errors
    pub fn set_json_str(&mut self, json: &str) -> Result<(), String> {
        self.json = serde_json::from_str(json).map_err(|e| e.to_string())?;
        Ok(())
    }
    /// updates a path from string and therefore can be some parsing errors
    pub fn set_path_str(&mut self, path: &str) -> Result<(), String> {
        self.path = Box::new(JsonPathInst::from_str(path)?);
        Ok(())
    }

    /// create a new instance from string and therefore can be some parsing errors
    pub fn from_str(json: &str, path: &str) -> Result<Self, String> {
        let json = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let path = Box::new(JsonPathInst::from_str(path)?);
        Ok(JsonPathFinder::new(json, path))
    }

    /// creates an instance to find a json slice from the json
    pub fn instance(&self) -> PathInstance {
        json_path_instance(&self.path.inner, &self.json)
    }
    /// finds a slice of data in the set json.
    /// The result is a vector of references to the incoming structure.
    pub fn find_slice(&self) -> Vec<JsonPathValue<'_, Value>> {
        self.instance().find((&(*self.json)).into())
    }

    /// finds a slice of data and wrap it with Value::Array by cloning the data.
    pub fn find(&self) -> Value {
        Value::Array(self.find_slice().into_iter().map(|v| v.to_data()).collect())
    }
}


#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use serde_json::{json, Value};
    use crate::{json_path_value, JsonPathFinder, JsonPathInst, JsonPathValue};
    use crate::JsonPathQuery;

    fn test(json: &str, path: &str, expected: Vec<JsonPathValue<Value>>) {
        match JsonPathFinder::from_str(json, path) {
            Ok(finder) => assert_eq!(finder.find_slice(), expected),
            Err(e) => panic!("error while parsing json or jsonpath: {}", e)
        }
    }

    fn template_json<'a>() -> &'a str {
        r#" {"store": { "book": [
             {
                 "category": "reference",
                 "author": "Nigel Rees",
                 "title": "Sayings of the Century",
                 "price": 8.95
             },
             {
                 "category": "fiction",
                 "author": "Evelyn Waugh",
                 "title": "Sword of Honour",
                 "price": 12.99
             },
             {
                 "category": "fiction",
                 "author": "Herman Melville",
                 "title": "Moby Dick",
                 "isbn": "0-553-21311-3",
                 "price": 8.99
             },
             {
                 "category": "fiction",
                 "author": "J. R. R. Tolkien",
                 "title": "The Lord of the Rings",
                 "isbn": "0-395-19395-8",
                 "price": 22.99
             }
         ],
         "bicycle": {
             "color": "red",
             "price": 19.95
         }
     },
     "array":[0,1,2,3,4,5,6,7,8,9],
     "orders":[
         {
             "ref":[1,2,3],
             "id":1,
             "filled": true
         },
         {
             "ref":[4,5,6],
             "id":2,
             "filled": false
         },
         {
             "ref":[7,8,9],
             "id":3,
             "filled": null
         }
      ],
     "expensive": 10 }"#
    }

    #[test]
    fn simple_test() {
        let j1 = json!(2);
        test("[1,2,3]", "$[1]", json_path_value![&j1,]);
    }

    #[test]
    fn root_test() {
        let js = serde_json::from_str(template_json()).unwrap();
        test(template_json(), "$", json_path_value![&js,]);
    }

    #[test]
    fn descent_test() {
        let v1 = json!("reference");
        let v2 = json!("fiction");
        test(template_json(), "$..category",
             json_path_value![
                 &v1,
                 &v2,
                 &v2,
                 &v2,
             ]);
        let js1 = json!(19.95);
        let js2 = json!(8.95);
        let js3 = json!(12.99);
        let js4 = json!(8.99);
        let js5 = json!(22.99);
        test(template_json(),
             "$.store..price",
             json_path_value![
                 &js1,
                 &js2,
                 &js3,
                 &js4,
                 &js5,
             ],
        );
        let js1 = json!("Nigel Rees");
        let js2 = json!("Evelyn Waugh");
        let js3 = json!("Herman Melville");
        let js4 = json!("J. R. R. Tolkien");
        test(template_json(),
             "$..author",
             json_path_value![
                 &js1,
                 &js2,
                 &js3,
                 &js4,
             ],
        );
    }

    #[test]
    fn wildcard_test() {
        let js1 = json!("reference");
        let js2 = json!("fiction");
        test(template_json(), "$..book.[*].category",
             json_path_value![
                 &js1,
                 &js2,
                 &js2,
                 &js2,
             ]);
        let js1 = json!("Nigel Rees");
        let js2 = json!("Evelyn Waugh");
        let js3 = json!("Herman Melville");
        let js4 = json!("J. R. R. Tolkien");
        test(template_json(),
             "$.store.book[*].author",
             json_path_value![
                 &js1,
                 &js2,
                 &js3,
                 &js4,
             ],
        );
    }

    #[test]
    fn field_test() {
        let value = json!({"active":1});
        test(r#"{"field":{"field":[{"active":1},{"passive":1}]}}"#,
             "$.field.field[?(@.active)]",
             json_path_value![&value,]);
    }

    #[test]
    fn index_index_test() {
        let value = json!("0-553-21311-3");
        test(template_json(), "$..book[2].isbn",
             json_path_value![
                 &value,
             ]);
    }

    #[test]
    fn index_unit_index_test() {
        let value = json!("0-553-21311-3");
        test(template_json(), "$..book[2,4].isbn",
             json_path_value![
                 &value,
             ]);
        let value1 = json!("0-395-19395-8");
        test(template_json(), "$..book[2,3].isbn",
             json_path_value![
                 &value,
                 &value1,
             ]);
    }

    #[test]
    fn index_unit_keys_test() {
        let js1 = json!("Moby Dick");
        let js2 = json!(8.99);
        let js3 = json!("The Lord of the Rings");
        let js4 = json!(22.99);
        test(template_json(), "$..book[2,3]['title','price']",
             json_path_value![
                 &js1,
                 &js2,
                 &js3,
                 &js4,
             ]);
    }

    #[test]
    fn index_slice_test() {
        let j0 = json!(0);
        let j1 = json!(1);
        let j2 = json!(2);
        let j3 = json!(3);
        let j4 = json!(4);
        let j5 = json!(5);
        let j6 = json!(6);
        let j7 = json!(7);
        let j8 = json!(8);
        let j9 = json!(9);
        test(template_json(),
             "$.array[:]",
             json_path_value![
                 &j0,
                 &j1,
                 &j2,
                 &j3,
                 &j4,
                 &j5,
                 &j6,
                 &j7,
                 &j8,
                 &j9,
             ]);
        test(template_json(),
             "$.array[1:4:2]",
             json_path_value![
                 &j1,
                 &j3,
             ]);
        test(template_json(),
             "$.array[::3]",
             json_path_value![
                 &j0,
                 &j3,
                 &j6,
                 &j9,
             ]);
        test(template_json(),
             "$.array[-1:]",
             json_path_value![
                 &j9,
             ]);
        test(template_json(),
             "$.array[-2:-1]",
             json_path_value![
                 &j8,
             ]);
    }

    #[test]
    fn index_filter_test() {
        let moby = json!("Moby Dick");
        let rings = json!("The Lord of the Rings");
        test(template_json(),
             "$..book[?(@.isbn)].title",
             json_path_value![
                 &moby,
                 &rings,
             ]);
        let sword = json!("Sword of Honour");
        test(template_json(),
             "$..book[?(@.price != 8.95)].title",
             json_path_value![
                 &sword,
                 &moby,
                 &rings,
             ]);
        let sayings = json!("Sayings of the Century");
        test(template_json(),
             "$..book[?(@.price == 8.95)].title",
             json_path_value![
                 &sayings,
             ]);
        let js895 = json!(8.95);
        test(template_json(),
             "$..book[?(@.author ~= '.*Rees')].price",
             json_path_value![&js895,]);
        let js12 = json!(12.99);
        let js899 = json!(8.99);
        let js2299 = json!(22.99);
        test(template_json(),
             "$..book[?(@.price >= 8.99)].price",
             json_path_value![
                 &js12,
                 &js899,
                 &js2299,
             ]);
        test(template_json(),
             "$..book[?(@.price > 8.99)].price",
             json_path_value![
                 &js12,
                 &js2299,
             ]);
        test(template_json(),
             "$..book[?(@.price < 8.99)].price",
             json_path_value![
                 &js895,
             ]);
        test(template_json(),
             "$..book[?(@.price <= 8.99)].price",
             json_path_value![
                 &js895,
                 &js899,
             ]);
        test(template_json(),
             "$..book[?(@.price <= $.expensive)].price",
             json_path_value![
                 &js895,
                 &js899,
             ]);
        test(template_json(),
             "$..book[?(@.price >= $.expensive)].price",
             json_path_value![
                 &js12,
                 &js2299,
             ]);
        test(template_json(),
             "$..book[?(@.title in ['Moby Dick','Shmoby Dick','Big Dick','Dicks'])].price",
             json_path_value![
                 &js899,
             ]);
        test(template_json(),
             "$..book[?(@.title nin ['Moby Dick','Shmoby Dick','Big Dick','Dicks'])].title",
             json_path_value![
                 &sayings,
                 &sword,
                 &rings,
             ]);
        test(template_json(),
             "$..book[?(@.author size 10)].title",
             json_path_value![
                 &sayings,
             ]);
        let filled_true = json!(1);
        test(template_json(),
             "$.orders[?(@.filled == true)].id",
             json_path_value![
                 &filled_true,
             ]);
        let filled_null = json!(3);
        test(template_json(),
            "$.orders[?(@.filled == null)].id",
            json_path_value![
                &filled_null,
            ]);
    }

    #[test]
    fn index_filter_sets_test() {
        let j1 = json!(1);
        test(template_json(),
             "$.orders[?(@.ref subsetOf [1,2,3,4])].id",
             json_path_value![
                 &j1,
             ]);
        let j2 = json!(2);
        test(template_json(),
             "$.orders[?(@.ref anyOf [1,4])].id",
             json_path_value![
                 &j1,
                 &j2,
             ]);
        let j3 = json!(3);
        test(template_json(),
             "$.orders[?(@.ref noneOf [3,6])].id",
             json_path_value![
                 &j3,
             ]);
    }

    #[test]
    fn query_test() {
        let json: Box<Value> = serde_json::from_str(template_json()).expect("to get json");
        let v = json.path("$..book[?(@.author size 10)].title")
            .expect("the path is correct");
        assert_eq!(v, json!(["Sayings of the Century"]));

        let json: Value = serde_json::from_str(template_json()).expect("to get json");
        let path = &json.path("$..book[?(@.author size 10)].title")
            .expect("the path is correct");

        assert_eq!(path, &json!(["Sayings of the Century"]));
    }

    #[test]
    fn find_slice_test() {
        let json: Box<Value> = serde_json::from_str(template_json()).expect("to get json");
        let path: Box<JsonPathInst> = Box::from(JsonPathInst::from_str("$..book[?(@.author size 10)].title")
            .expect("the path is correct"));
        let finder = JsonPathFinder::new(json, path);

        let v = finder.find_slice();
        let js = json!("Sayings of the Century");
        assert_eq!(v, json_path_value![&js,]);
    }

    #[test]
    fn find_in_array_test() {
        let json: Box<Value> = Box::new(json!([{"verb": "TEST"}, {"verb": "RUN"}]));
        let path: Box<JsonPathInst> = Box::from(
            JsonPathInst::from_str("$.[?(@.verb == 'TEST')]").expect("the path is correct")
        );
        let finder = JsonPathFinder::new(json, path);

        let v = finder.find_slice();
        let js = json!({"verb":"TEST"});
        assert_eq!(v, json_path_value![&js,]);
    }
    #[test]
    fn length_test() {
        let json: Box<Value> = Box::new(json!([{"verb": "TEST"},{"verb": "TEST"}, {"verb": "RUN"}]));
        let path: Box<JsonPathInst> = Box::from(
            JsonPathInst::from_str("$.[?(@.verb == 'TEST')].length()").expect("the path is correct")
        );
        let finder = JsonPathFinder::new(json, path);

        let v = finder.find();
        let js = json!([2]);
        assert_eq!(v, js);

        let json: Box<Value> = Box::new(json!([{"verb": "TEST"},{"verb": "TEST"}, {"verb": "RUN"}]));
        let path: Box<JsonPathInst> = Box::from(
            JsonPathInst::from_str("$.length()").expect("the path is correct")
        );
        let finder = JsonPathFinder::new(json, path);
        assert_eq!(finder.find(), json!([3]));


        let json: Box<Value> = Box::new(json!([{"verb": "TEST"},{"verb": "TEST","x":3}, {"verb": "RUN"}]));
        let path: Box<JsonPathInst> = Box::from(
            JsonPathInst::from_str("$.[?(@.verb == 'TEST')].[*].length()").expect("the path is correct")
        );
        let finder = JsonPathFinder::new(json, path);
        assert_eq!(finder.find(), json!([3]));



        let json: Box<Value> = Box::new(json!({"verb": "TEST"}));
        let path: Box<JsonPathInst> = Box::from(
            JsonPathInst::from_str("$.length()").expect("the path is correct")
        );
        let finder = JsonPathFinder::new(json, path);
        assert_eq!(finder.find(), json!([0]));

        let json: Box<Value> = Box::new(json!(1));
        let path: Box<JsonPathInst> = Box::from(
            JsonPathInst::from_str("$.length()").expect("the path is correct")
        );
        let finder = JsonPathFinder::new(json, path);
        assert_eq!(finder.find(), json!([0]));

        let json: Box<Value> = Box::new(json!([[1],[2],[3]]));
        let path: Box<JsonPathInst> = Box::from(
            JsonPathInst::from_str("$.length()").expect("the path is correct")
        );
        let finder = JsonPathFinder::new(json, path);
        assert_eq!(finder.find(), json!([3]));
    }
}