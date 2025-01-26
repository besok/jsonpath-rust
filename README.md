# jsonpath-rust

[![Crates.io](https://img.shields.io/crates/v/jsonpath-rust)](https://crates.io/crates/jsonpath-rust)
[![docs.rs](https://img.shields.io/docsrs/jsonpath-rust)](https://docs.rs/jsonpath-rust/latest/jsonpath_rust)
[![Rust CI](https://github.com/besok/jsonpath-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/besok/jsonpath-rust/actions/workflows/ci.yml)

The library provides the basic functionality to find the set of the data according to the filtering query. The idea
comes from XPath for XML structures. The details can be found [there](https://goessner.net/articles/JsonPath/)
Therefore JsonPath is a query language for JSON, similar to XPath for XML. The JsonPath query is a set of assertions to
specify the JSON fields that need to be verified.

Python bindings ([jsonpath-rust-bindings](https://github.com/night-crawler/jsonpath-rust-bindings)) are available on
pypi:

```bash
pip install jsonpath-rust-bindings
```

## The compliance with RFC 9535
The library is compliant with the [RFC 9535](https://datatracker.ietf.org/doc/html/rfc9535) 

## Simple examples

Let's suppose we have a following json:

```json
{
  "shop": {
    "orders": [
      {
        "id": 1,
        "active": true
      },
      {
        "id": 2
      },
      {
        "id": 3
      },
      {
        "id": 4,
        "active": true
      }
    ]
  }
}
 ```

And we pursue to find all orders id having the field 'active'. We can construct the jsonpath instance like
that  ```$.shop.orders[?(@.active)].id``` and get the result ``` [1,4] ```

## The jsonpath description

### Functions

#### Size

A function `length()` transforms the output of the filtered expression into a size of this element
It works with arrays, therefore it returns a length of a given array, otherwise null.

`$.some_field.length()`

**To use it** for objects, the operator `[*]` can be used.
`$.object.[*].length()`

### Operators

| Operator                   | Description                                                                                                                                                  | Where to use                                                                                                                                |
|----------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------|
| `$`                        | Pointer to the root of the json.                                                                                                                             | It is gently advising to start every jsonpath from the root. Also, inside the filters to point out that the path is starting from the root. |
| `@`                        | Pointer to the current element inside the filter operations.                                                                                                 | It is used inside the filter operations to iterate the collection.                                                                          |
| `*` or `[*]`               | Wildcard. It brings to the list all objects and elements regardless their names.                                                                             | It is analogue a flatmap operation.                                                                                                         |
| `<..>`                     | Descent operation. It brings to the list all objects, children of that objects and etc                                                                       | It is analogue a flatmap operation.                                                                                                         |
| `.<name>` or `.['<name>']` | the key pointing to the field of the object                                                                                                                  | It is used to obtain the specific field.                                                                                                    |
| `['<name>' (, '<name>')]`  | the list of keys                                                                                                                                             | the same usage as for a single key but for list                                                                                             |
| `[<number>]`               | the filter getting the element by its index.                                                                                                                 |                                                                                                                                             |
| `[<number> (, <number>)]`  | the list if elements of array according to their indexes representing these numbers.                                                                         |                                                                                                                                             |
| `[<start>:<end>:<step>]`   | slice operator to get a list of element operating with their indexes. By default step = 1, start = 0, end = array len. The elements can be omitted ```[:]``` |                                                                                                                                             |
| `[?(<expression>)]`        | the logical expression to filter elements in the list.                                                                                                       | It is used with arrays preliminary.                                                                                                         |

### Filter expressions

The expressions appear in the filter operator like that `[?(@.len > 0)]`. The expression in general consists of the
following elements:

- Left and right operands, that is ,in turn, can be a static value,representing as a primitive type like a number,
  string value `'value'`, array of them or another json path instance.
- Expression sign, denoting what action can be performed

| Expression sign | Description                                                                                | Where to use                                                                                             |
|-----------------|--------------------------------------------------------------------------------------------|----------------------------------------------------------------------------------------------------------|
| `!`             | Not                                                                                        | To negate the expression                                                                                 |
| `==`            | Equal                                                                                      | To compare numbers or string literals                                                                    |
| `!=`            | Unequal                                                                                    | To compare numbers or string literals in opposite way to equals                                          |
| `<`             | Less                                                                                       | To compare numbers                                                                                       |
| `>`             | Greater                                                                                    | To compare numbers                                                                                       |
| `<=`            | Less or equal                                                                              | To compare numbers                                                                                       |
| `>=`            | Greater or equal                                                                           | To compare numbers                                                                                       |
| `~=`            | Regular expression                                                                         | To find the incoming right side in the left side.                                                        |
| `in`            | Find left element in the list of right elements.                                           |                                                                                                          |
| `nin`           | The same one as saying above but carrying the opposite sense.                              |                                                                                                          |
| `size`          | The size of array on the left size should be corresponded to the number on the right side. |                                                                                                          |
| `noneOf`        | The left size has no intersection with right                                               |                                                                                                          |
| `anyOf`         | The left size has at least one intersection with right                                     |                                                                                                          |
| `subsetOf`      | The left is a subset of the right side                                                     |                                                                                                          |
| `?`             | Exists operator.                                                                           | The operator checks the existence of the field depicted on the left side like that `[?(@.key.isActive)]` |

Filter expressions can be chained using `||` and `&&` (logical or and logical and correspondingly) in the following way:

```json
{
  "key": [
    {
      "city": "London",
      "capital": true,
      "size": "big"
    },
    {
      "city": "Berlin",
      "capital": true,
      "size": "big"
    },
    {
      "city": "Tokyo",
      "capital": true,
      "size": "big"
    },
    {
      "city": "Moscow",
      "capital": true,
      "size": "big"
    },
    {
      "city": "Athlon",
      "capital": false,
      "size": "small"
    },
    {
      "city": "Dortmund",
      "capital": false,
      "size": "big"
    },
    {
      "city": "Dublin",
      "capital": true,
      "size": "small"
    }
  ]
}
```

The path ``` $.key[?(@.capital == false || @size == 'small')].city ``` will give the following result:

```json
[
  "Athlon",
  "Dublin",
  "Dortmund"
]
```

And the path ``` $.key[?(@.capital == false && @size != 'small')].city ``` ,in its turn, will give the following result:

```json
[
  "Dortmund"
]
```

By default, the operators have the different priority so `&&` has a higher priority so to change it the brackets can be
used.
``` $.[?((@.f == 0 || @.f == 1) && ($.x == 15))].city ```

## Examples

Given the json

 ```json
{
  "store": {
    "book": [
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
  "expensive": 10
}
 ```

| JsonPath                             | Result                                                       |
|--------------------------------------|:-------------------------------------------------------------|
| `$.store.book[*].author`             | The authors of all books                                     |
| `$..book[?(@.isbn)]`                 | All books with an ISBN number                                |
| `$.store.*`                          | All things, both books and bicycles                          |
| `$..author`                          | All authors                                                  |
| `$.store..price`                     | The price of everything                                      |
| `$..book[2]`                         | The third book                                               |
| `$..book[-2]`                        | The second to last book                                      |
| `$..book[0,1]`                       | The first two books                                          |
| `$..book[:2]`                        | All books from index 0 (inclusive) until index 2 (exclusive) |
| `$..book[1:2]`                       | All books from index 1 (inclusive) until index 2 (exclusive) |
| `$..book[-2:]`                       | Last two books                                               |
| `$..book[2:]`                        | Book number two from tail                                    |
| `$.store.book[?(@.price < 10)]`      | All books in store cheaper than 10                           |
| `$..book[?(@.price <= $.expensive)]` | All books in store that are not "expensive"                  |
| `$..book[?(@.author ~= '(?i)REES')]` | All books matching regex (ignore case)                       |
| `$..*`                               | Give me every thing                                          |

## Library Usage

The library intends to provide the basic functionality for ability to find the slices of data using the syntax, saying
above. The dependency can be found as following:
``` jsonpath-rust = *```

The basic example is the following one:

The library returns a `json path value` as a result.
This is enum type which represents:

- `Slice` - a point to the passed original json
- `NewValue` - a new json data that has been generated during the path( for instance length operator)
- `NoValue` - indicates there is no match between given json and jsonpath in the most cases due to absent fields or
  inconsistent data.

To extract data there are two methods, provided on the `value`:

```rust
let v:JsonPathValue<Value> =...
v.to_data();
v.slice_or( & some_dafault_value)
```

### Find

there are 4 different functions to find data inside a `value`.
All take references, to increase reusability. Especially json parsing and jsonpath parsing can take significant time,
compared to a simple find.

The methods `find`, `find_as_path`, `find_slice` and `find_slice_ptr` take the same inputs, but handle them differently
depending on your usecase. They are further described in
the [docs](https://docs.rs/jsonpath-rust/latest/jsonpath_rust/enum.JsonPath.html#implementations).

```rust
use jsonpath_rust::{JsonPath, JsonPathValue};
use serde_json::json;
use std::str::FromStr;

fn main() {
    let data = json!({"first":{"second":[{"active":1},{"passive":1}]}});
    let path = JsonPath::from_str("$.first.second[?(@.active)]").unwrap();
    let slice_of_data = path.find_slice(&data);

    let expected_value = json!({"active":1});
    let expected_path = "$.['first'].['second'][0]".to_string();

    assert_eq!(
        slice_of_data,
        vec![JsonPathValue::Slice(&expected_value, expected_path)]
    );
}
```

### The structure

The internal structure of the `JsonPath` can be found here:
https://docs.rs/jsonpath-rust/latest/jsonpath_rust/parser/model/enum.JsonPath.html

The internal structure of the `JsonPathIndex` can be found here:
https://docs.rs/jsonpath-rust/latest/jsonpath_rust/parser/model/enum.JsonPathIndex.html

### JsonLike

The library provides a trait `JsonLike` that can be implemented for any type.
This allows you to use the `JsonPath` methods on your own types.

### Update the JsonLike structure by path

The library does not provide the functionality to update the json structure in the query itself.
Instead, the library provides the ability to update the json structure by the path.
Thus, the user needs to find a path for the `JsonLike` structure and update it manually.

There are two methods in the `JsonLike` trait:

- `reference_mut` - returns a mutable reference to the element by the path
- `reference` - returns a reference to the element by the path
  They accept a `JsonPath` instance and return a `Option<&mut Value>` or `Option<&Value>` respectively.
  The path is supported with the limited elements namely only the elements with the direct access:
- root
- field
- index
  The path can be obtained manually or `find_as_path` method can be used.

```rust
#[test]
fn update_by_path_test() -> Result<(), JsonPathParserError> {
    let mut json = json!([
            {"verb": "RUN","distance":[1]},
            {"verb": "TEST"},
            {"verb": "DO NOT RUN"}
        ]);

    let path: Box<JsonPath> = Box::from(JsonPath::try_from("$.[?(@.verb == 'RUN')]")?);
    let elem = path
        .find_as_path(&json)
        .get(0)
        .cloned()
        .ok_or(JsonPathParserError::InvalidJsonPath("".to_string()))?;

    if let Some(v) = json
        .reference_mut(elem)?
        .and_then(|v| v.as_object_mut())
        .and_then(|v| v.get_mut("distance"))
        .and_then(|v| v.as_array_mut())
    {
        v.push(json!(2))
    }

    assert_eq!(
        json,
        json!([
                {"verb": "RUN","distance":[1,2]},
                {"verb": "TEST"},
                {"verb": "DO NOT RUN"}
            ])
    );

    Ok(())
}
```

## How to contribute

TBD

## How to update version

- update files
- commit them
- add tag `git tag -a v<Version> -m "message"`
- git push origin <tag_name>
