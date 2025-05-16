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
//!
//! [`there`]: https://goessner.net/articles/JsonPath/
#![allow(warnings)]

pub mod query;

#[allow(clippy::module_inception)]
pub mod parser;

pub use jsonpath_ast::{parse, ast};

#[macro_use]
extern crate pest_derive;
extern crate core;
extern crate pest;

use crate::query::queryable::Queryable;
use crate::query::{Queried, QueryPath, QueryRef};
use serde_json::Value;

/// A trait for types that can be queried with JSONPath.
pub trait JsonPath: Queryable {
    /// Queries the value with a JSONPath expression and returns a vector of `QueryResult`.
    fn query_with_path(&self, path: &str) -> Queried<Vec<QueryRef<Self>>> {
        query::js_path(path, self)
    }

    /// Queries the value with a JSONPath expression and returns a vector of values.
    fn query_only_path(&self, path: &str) -> Queried<Vec<QueryPath>> {
        query::js_path_path(path, self)
    }

    /// Queries the value with a JSONPath expression and returns a vector of values, omitting the path.
    fn query(&self, path: &str) -> Queried<Vec<&Self>> {
        query::js_path_vals(path, self)
    }
}

impl JsonPath for Value {}
