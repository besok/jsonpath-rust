# jsonpath-rust

[![Crates.io](https://img.shields.io/crates/v/jsonpath-rust)](https://crates.io/crates/jsonpath-rust)
[![docs.rs](https://img.shields.io/docsrs/jsonpath-rust)](https://docs.rs/jsonpath-rust/latest/jsonpath_rust)
[![Rust CI](https://github.com/besok/jsonpath-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/besok/jsonpath-rust/actions/workflows/ci.yml)

The library provides the extensive functionality to find data sets according to filtering queries.
Inspired by XPath for XML structures, JsonPath is a query language for JSON.
The specification is described in [RFC 9535](https://www.rfc-editor.org/rfc/rfc9535.html).

# Important note

The version 1.0.0 has a breaking change. The library has been rewritten from scratch to provide compliance with the RFC9535.

The changes are:

- The library is now fully compliant with the RFC 9535.
- New structures and apis were introduced to provide the compliance with the RFC 9535.
  - `Queryable` instead of `JsonLike`
  - `Queried<Queryable>` instead of  `Result<Value, JsonPathParserError>`
  - `JsonPath#{query_with_path, query_only_path, query}` to operate with the `Queryable` structure
  - `JsonPathError` instead of `JsonPathParserError`
  - `QueryRef` to provide the reference to the value and path
- The functions in, nin, noneOf, anyOf, subsetOf are now implemented as custom filter expressions and renamed to `in`,
  `nin`, `none_of`, `any_of`, `subset_of` respectively.
- The function length was removed (the size can be checked using rust native functions for using it in filter there is length expression).

## The compliance with RFC 9535

The library is fully compliant (except several cases) with the standard [RFC 9535](https://www.rfc-editor.org/rfc/rfc9535.html)
To check the compliance with the standard, please be headed to [rfc9535 subfolder](rfc9535/README.md)


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

| JsonPath                           | Result                                                       |
|------------------------------------|:-------------------------------------------------------------|
| `$.store.book[*].author`           | The authors of all books                                     |
| `$..book[?@.isbn]`                 | All books with an ISBN number                                |
| `$.store.*`                        | All things, both books and bicycles                          |
| `$..author`                        | All authors                                                  |
| `$.store..price`                   | The price of everything                                      |
| `$..book[2]`                       | The third book                                               |
| `$..book[-2]`                      | The second to last book                                      |
| `$..book[0,1]`                     | The first two books                                          |
| `$..book[:2]`                      | All books from index 0 (inclusive) until index 2 (exclusive) |
| `$..book[1:2]`                     | All books from index 1 (inclusive) until index 2 (exclusive) |
| `$..book[-2:]`                     | Last two books                                               |
| `$..book[2:]`                      | Book number two from tail                                    |
| `$.store.book[?@.price < 10]`      | All books in store cheaper than 10                           |
| `$..book[?@.price <= $.expensive]` | All books in store that are not "expensive"                  |
| `$..book[?@.author ~= '(?i)REES']` | All books matching regex (ignore case)                       |
| `$..*`                             | Give me every thing                                          |

## Library Usage

### Extensions
The library provides the following extensions:

- **in**  
Checks if the first argument is in the array provided as the second argument. Example: `$.elems[?in(@, $.list)]` 
Returns elements from `$.elems` that are present in `$.list`.

- **nin**  
Checks if the first argument is not in the array provided as the second argument. Example: `$.elems[?nin(@, $.list)]`
Returns elements from `$.elems` that are not present in `$.list`.

- **none_of**  
Checks if none of the elements in the first array are in the second array. Example: `$.elems[?none_of(@, $.list)]` 
Returns arrays from `$.elems` that have no elements in common with `$.list`.

- **any_of**  
Checks if any of the elements in the first array are in the second array. Example: `$.elems[?any_of(@, $.list)]` 
Returns arrays from `$.elems` that have at least one element in common with `$.list`.

- **subset_of**  
Checks if all elements in the first array are in the second array. Example: `$.elems[?subset_of(@, $.list)]` 
Returns arrays from `$.elems` where all elements are present in `$.list`.


### Queryable

The library provides a trait `Queryable` that can be implemented for any type.
This allows you to use the `JsonPath` methods on your own types.

### Queried with path

```rust

fn union() -> Queried<()> {
    let json = json!([0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

    // QueryRes is a tuple of (value, path) for references and just value for owned values
    let vec: Vec<QueryRef<Value>> = json.query_with_path("$[1,5:7]")?;
    assert_eq!(
        vec,
        vec![
            (&json!(1), "$[1]".to_string()).into(),
            (&json!(5), "$[5]".to_string()).into(),
            (&json!(6), "$[6]".to_string()).into(),
        ]
    );

    Ok(())
}

```

### Queried without path

```rust
fn exp_no_error() -> Queried<()> {
    let json = json!([
      {
        "a": 100,
        "d": "e"
      },
      {
        "a": 100.1,
        "d": "f"
      },
      {
        "a": "100",
        "d": "g"
      }
    ]);

    let vec: Vec<&Value> = json.query("$[?@.a==1E2]")?;
    assert_eq!(
        vec.iter().collect::<Vec<_>>(),
        vec![&json!({"a":100, "d":"e"})]
    );

    Ok(())
}
```

### Queried with only path

```rust
fn filter_data() -> Queried<()> {
    let json = json!({
      "a": 1,
      "b": 2,
      "c": 3
    });

    let vec: Vec<String> = json
        .query_only_path("$[?@<3]")?
        .into_iter()
        .map(Option::unwrap_or_default)
        .collect();

    assert_eq!(vec, vec!["$['a']".to_string(), "$['b']".to_string()]);

    Ok(())
}
```

### Update the Queryable structure by path

The library does not provide the functionality to update the json structure in the query itself.
Instead, the library provides the ability to update the json structure by the path.
Thus, the user needs to find a path for the `JsonLike` structure and update it manually.

There are two methods in the `Queryable` trait:

- `reference_mut` - returns a mutable reference to the element by the path
- `reference` - returns a reference to the element by the path

They accept a `JsonPath` instance and return a `Option<&mut Self>` or `Option<&Self>` respectively.

The path is supported with the limited elements namely only the elements with the direct access:

- root
- field
- index

```rust
 fn update_by_path_test() -> Queried<()> {
    let mut json = json!([
            {"verb": "RUN","distance":[1]},
            {"verb": "TEST"},
            {"verb": "DO NOT RUN"}
        ]);

    let path = json.query_only_path("$.[?(@.verb == 'RUN')]")?;
    let elem = path.first().unwrap_or_default();

    if let Some(v) = json
        .reference_mut(elem)
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

### Python bindings

Python bindings ([jsonpath-rust-bindings](https://github.com/night-crawler/jsonpath-rust-bindings)) are available on
pypi:

```bash
pip install jsonpath-rust-bindings
```

## How to contribute

TBD

## How to update version

- update files
- commit them
- add tag `git tag -a v<Version> -m "message"`
- git push origin <tag_name>
