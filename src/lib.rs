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
//! use self::jsonpath_rust::JsonPathFinder;
//! fn test(){
//!     let finder = JsonPathFinder::from_str(r#"{"first":{"second":[{"active":1},{"passive":1}]}}"#, "$.first.second[?(@.active)]").unwrap();
//!     let slice_of_data:Vec<&Value> = finder.find();
//!     assert_eq!(slice_of_data, vec![&json!({"active":1})]);
//! }
//! ```
//! or even simpler:
//!
//! ```
//! use serde_json::{json,Value};
//! use self::jsonpath_rust::JsonPathFinder;
//!
//! fn test(json: &str, path: &str, expected: Vec<&Value>) {
//!    match JsonPathFinder::from_str(json, path) {
//!        Ok(finder) => assert_eq!(finder.find(), expected),
//!        Err(e) => panic!("error while parsing json or jsonpath: {}", e)
//!    }
//! }
//! ```
//!
//! [`there`]: https://goessner.net/articles/JsonPath/


use serde_json::{Value, from_str};
use crate::parser::parser::parse_json_path;
use crate::path::{json_path_instance, PathInstance};
use crate::parser::model::JsonPath;

mod parser;
mod path;


#[macro_use]
extern crate pest_derive;
extern crate pest;

/// The base structure conjuncting the json instance and jsonpath instance
pub struct JsonPathFinder {
    json: Value,
    path: JsonPath,
}

impl JsonPathFinder {
    /// creates a new instance of [[JsonPathFinder]]
    pub fn new(json: Value, path: JsonPath) -> Self {
        JsonPathFinder { json, path }
    }

    /// updates a path with a new one
    pub fn set_path(&mut self, path: JsonPath) {
        self.path = path
    }
    /// updates a json with a new one
    pub fn set_json(&mut self, json: Value) {
        self.json = json
    }
    /// updates a json from string and therefore can be some parsing errors
    pub fn set_json_str(&mut self, json: &str) -> Result<(), String> {
        self.json = serde_json::from_str(json).map_err(|e| e.to_string())?;
        Ok(())
    }
    /// updates a path from string and therefore can be some parsing errors
    pub fn set_path_str(&mut self, path: &str) -> Result<(), String> {
        self.path = parse_json_path(path).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// create a new instance from string and therefore can be some parsing errors
    pub fn from_str(json: &str, path: &str) -> Result<Self, String> {
        let json = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let path = parse_json_path(path).map_err(|e| e.to_string())?;
        Ok(JsonPathFinder::new(json, path))
    }

    /// creates an instance to find a json slice from the json
    pub fn instance(&self) -> PathInstance {
        json_path_instance(&self.path, &self.json)
    }
    /// finds a slice of data in the set json.
    /// The result is a vector of references to the incoming structure.
    pub fn find(&self) -> Vec<&Value> {
        self.instance().find(&self.json)
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::parser::parse_json_path;
    use serde_json::{json, Value};
    use crate::path::json_path_instance;
    use crate::JsonPathFinder;

    fn test(json: &str, path: &str, expected: Vec<&Value>) {
        match JsonPathFinder::from_str(json, path) {
            Ok(finder) => assert_eq!(finder.find(), expected),
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
     "orders":[{"ref":[1,2,3],"id":1},{"ref":[4,5,6],"id":2},{"ref":[7,8,9],"id":3}],
     "expensive": 10 }"#
    }

    #[test]
    fn simple_test() {
        test("[1,2,3]", "$[1]", vec![&json!(2)]);
    }

    #[test]
    fn root_test() {
        test(template_json(), "$", vec![&serde_json::from_str(template_json()).unwrap()]);
    }

    #[test]
    fn descent_test() {
        test(template_json(), "$..category",
             vec![
                 &json!("reference"),
                 &json!("fiction"),
                 &json!("fiction"),
                 &json!("fiction"),
             ]);
        test(template_json(),
             "$.store..price",
             vec![
                 &json!(19.95),
                 &json!(8.95),
                 &json!(12.99),
                 &json!(8.99),
                 &json!(22.99),
             ],
        );
        test(template_json(),
             "$..author",
             vec![
                 &json!("Nigel Rees"),
                 &json!("Evelyn Waugh"),
                 &json!("Herman Melville"),
                 &json!("J. R. R. Tolkien")
             ],
        );
    }

    #[test]
    fn wildcard_test() {
        test(template_json(), "$..book.[*].category",
             vec![
                 &json!("reference"),
                 &json!("fiction"),
                 &json!("fiction"),
                 &json!("fiction"),
             ]);
        test(template_json(),
             "$.store.book[*].author",
             vec![
                 &json!("Nigel Rees"),
                 &json!("Evelyn Waugh"),
                 &json!("Herman Melville"),
                 &json!("J. R. R. Tolkien")
             ],
        );
    }

    #[test]
    fn field_test() {
        test(r#"{"field":{"field":[{"active":1},{"passive":1}]}}"#,
             "$.field.field[?(@.active)]",
             vec![&json!({"active":1})]);
    }

    #[test]
    fn index_index_test() {
        test(template_json(), "$..book[2].isbn",
             vec![
                 &json!("0-553-21311-3"),
             ]);
    }

    #[test]
    fn index_unit_index_test() {
        test(template_json(), "$..book[2,4].isbn",
             vec![
                 &json!("0-553-21311-3"),
             ]);
        test(template_json(), "$..book[2,3].isbn",
             vec![
                 &json!("0-553-21311-3"),
                 &json!("0-395-19395-8"),
             ]);
    }

    #[test]
    fn index_unit_keys_test() {
        test(template_json(), "$..book[2,3]['title','price']",
             vec![
                 &json!("Moby Dick"),
                 &json!(8.99),
                 &json!("The Lord of the Rings"),
                 &json!(22.99),
             ]);
    }

    #[test]
    fn index_slice_test() {
        test(template_json(),
             "$.array[:]",
             vec![
                 &json!(0),
                 &json!(1),
                 &json!(2),
                 &json!(3),
                 &json!(4),
                 &json!(5),
                 &json!(6),
                 &json!(7),
                 &json!(8),
                 &json!(9),
             ]);
        test(template_json(),
             "$.array[1:4:2]",
             vec![
                 &json!(1),
                 &json!(3),
             ]);
        test(template_json(),
             "$.array[::3]",
             vec![
                 &json!(0),
                 &json!(3),
                 &json!(6),
                 &json!(9),
             ]);
        test(template_json(),
             "$.array[-1:]",
             vec![
                 &json!(9),
             ]);
        test(template_json(),
             "$.array[-2:-1]",
             vec![
                 &json!(8),
             ]);
    }

    #[test]
    fn index_filter_test() {
        test(template_json(),
             "$..book[?(@.isbn)].title",
             vec![
                 &json!("Moby Dick"),
                 &json!("The Lord of the Rings"),
             ]);
        test(template_json(),
             "$..book[?(@.price != 8.95)].title",
             vec![
                 &json!("Sword of Honour"),
                 &json!("Moby Dick"),
                 &json!("The Lord of the Rings"),
             ]);
        test(template_json(),
             "$..book[?(@.price == 8.95)].title",
             vec![
                 &json!("Sayings of the Century"),
             ]);
        test(template_json(),
             "$..book[?(@.author ~= '.*Rees')].price",
             vec![
                 &json!(8.95),
             ]);
        test(template_json(),
             "$..book[?(@.price >= 8.99)].price",
             vec![
                 &json!(12.99),
                 &json!(8.99),
                 &json!(22.99),
             ]);
        test(template_json(),
             "$..book[?(@.price > 8.99)].price",
             vec![
                 &json!(12.99),
                 &json!(22.99),
             ]);
        test(template_json(),
             "$..book[?(@.price < 8.99)].price",
             vec![
                 &json!(8.95),
             ]);
        test(template_json(),
             "$..book[?(@.price <= 8.99)].price",
             vec![
                 &json!(8.95),
                 &json!(8.99),
             ]);
        test(template_json(),
             "$..book[?(@.price <= $.expensive)].price",
             vec![
                 &json!(8.95),
                 &json!(8.99),
             ]);
        test(template_json(),
             "$..book[?(@.price >= $.expensive)].price",
             vec![
                 &json!(12.99),
                 &json!(22.99),
             ]);
        test(template_json(),
             "$..book[?(@.title in ['Moby Dick','Shmoby Dick','Big Dick','Dicks'])].price",
             vec![
                 &json!(8.99),
             ]);
        test(template_json(),
             "$..book[?(@.title nin ['Moby Dick','Shmoby Dick','Big Dick','Dicks'])].title",
             vec![
                 &json!("Sayings of the Century"),
                 &json!("Sword of Honour"),
                 &json!("The Lord of the Rings"),
             ]);
        test(template_json(),
             "$..book[?(@.author size 10)].title",
             vec![
                 &json!("Sayings of the Century"),
             ]);
    }

    #[test]
    fn index_filter_sets_test() {
        test(template_json(),
             "$.orders[?(@.ref subsetOf [1,2,3,4])].id",
             vec![
                 &json!(1),
             ]);
        test(template_json(),
             "$.orders[?(@.ref anyOf [1,4])].id",
             vec![
                 &json!(1),
                 &json!(2),
             ]);
        test(template_json(),
             "$.orders[?(@.ref noneOf [3,6])].id",
             vec![
                 &json!(3),
             ]);
    }
}