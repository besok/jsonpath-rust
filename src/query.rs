mod atom;
mod comparable;
mod comparison;
mod filter;
mod jp_query;
pub mod queryable;
mod segment;
mod selector;
pub mod state;
mod test;
mod test_function;

use crate::parser::errors::JsonPathError;
use crate::parser::parse_json_path;
use crate::query::queryable::Queryable;
use crate::query::state::{Data, Pointer};
use state::State;
use std::borrow::Cow;

/// A type that can be queried with JSONPath, typically string
pub type QueryPath = String;

/// A type that can be queried with JSONPath, typically Result
pub type Queried<T> = Result<T, JsonPathError>;

/// Main internal trait to implement the logic of processing jsonpath.
pub trait Query {
    fn process<'a, T: Queryable>(&self, state: State<'a, T>) -> State<'a, T>;
}

/// The resulting type of JSONPath query.
/// It can either be a value or a reference to a value with its path.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryRef<'a, T: Queryable>(&'a T, QueryPath);

impl<'a, T: Queryable> From<(&'a T, QueryPath)> for QueryRef<'a, T> {
    fn from((inner, path): (&'a T, QueryPath)) -> Self {
        QueryRef(inner, path)
    }
}
impl<'a, T: Queryable> From<(&'a T, &str)> for QueryRef<'a, T> {
    fn from((inner, path): (&'a T, &str)) -> Self {
        QueryRef(inner, path.to_string())
    }
}

impl<'a, T: Queryable> QueryRef<'a, T> {
    pub fn val(self) -> &'a T {
        self.0
    }
    pub fn path(self) -> QueryPath {
        self.1
    }
}

impl<'a, T: Queryable> From<Pointer<'a, T>> for QueryRef<'a, T> {
    fn from(pointer: Pointer<'a, T>) -> Self {
        QueryRef(pointer.inner, pointer.path)
    }
}

/// The main function to process a JSONPath query.
/// It takes a path and a value, and returns a vector of `QueryResult` thus values + paths.
pub fn js_path<'a, T: Queryable>(path: &str, value: &'a T) -> Queried<Vec<QueryRef<'a, T>>> {
    match parse_json_path(path)?.process(State::root(value)).data {
        Data::Ref(p) => Ok(vec![p.into()]),
        Data::Refs(refs) => Ok(refs.into_iter().map(Into::into).collect()),
        Data::Value(v) => Err(v.into()),
        Data::Nothing => Ok(vec![]),
    }
}

/// A convenience function to process a JSONPath query and return a vector of values, omitting the path.
pub fn js_path_vals<'a, T: Queryable>(path: &str, value: &'a T) -> Queried<Vec<&'a T>> {
    Ok(js_path(path, value)?
        .into_iter()
        .map(|r| r.val())
        .collect::<Vec<_>>())
}

/// A convenience function to process a JSONPath query and return a vector of paths, omitting the values.
pub fn js_path_path<T: Queryable>(path: &str, value: &T) -> Queried<Vec<QueryPath>> {
    Ok(js_path(path, value)?
        .into_iter()
        .map(|r| r.path())
        .collect::<Vec<_>>())
}

#[cfg(test)]
mod tests {
    use crate::parser::errors::JsonPathError;
    use crate::parser::Parsed;
    use crate::query::queryable::Queryable;
    use crate::query::{js_path, Queried, QueryRef};
    use crate::JsonPath;
    use serde_json::{json, Value};

    fn test<'a, R>(json: &'a str, path: &str, expected: Vec<R>) -> Parsed<()>
    where
        R: Into<QueryRef<'a, Value>>,
    {
        let json: Value = serde_json::from_str(json).map_err(|v| JsonPathError::NoRulePath)?;
        let expected: Vec<QueryRef<'a, Value>> = expected.into_iter().map(|v| v.into()).collect();
        assert_eq!(json.query_with_path(path)?, expected);

        Ok(())
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
    fn update_by_path_test() -> Queried<()> {
        let mut json = json!([
            {"verb": "RUN","distance":[1]},
            {"verb": "TEST"},
            {"verb": "DO NOT RUN"}
        ]);

        let path = json.query_only_path("$.[?(@.verb == 'RUN')]")?;
        let elem = path.first().cloned().unwrap_or_default();

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

    #[test]
    fn simple_test() {
        let j1 = json!(2);
        let _ = test("[1,2,3]", "$[1]", vec![(&j1, "$[1]".to_string())]);
    }

    #[test]
    fn root_test() {
        let js = serde_json::from_str(template_json()).unwrap();
        let _ = test(template_json(), "$", vec![(&js, "$")]);
    }

    #[test]
    fn descent_test() {
        let v1 = json!("reference");
        let v2 = json!("fiction");
        let _ = test(
            template_json(),
            "$..category",
            vec![
                (&v1, "$['store']['book'][0]['category']"),
                (&v2, "$['store']['book'][1]['category']"),
                (&v2, "$['store']['book'][2]['category']"),
                (&v2, "$['store']['book'][3]['category']"),
            ],
        );
        let js1 = json!(19.95);
        let js2 = json!(8.95);
        let js3 = json!(12.99);
        let js4 = json!(8.99);
        let js5 = json!(22.99);
        let _ = test(
            template_json(),
            "$.store..price",
            vec![
                (&js1, "$['store']['bicycle']['price']"),
                (&js2, "$['store']['book'][0]['price']"),
                (&js3, "$['store']['book'][1]['price']"),
                (&js4, "$['store']['book'][2]['price']"),
                (&js5, "$['store']['book'][3]['price']"),
            ],
        );
        let js1 = json!("Nigel Rees");
        let js2 = json!("Evelyn Waugh");
        let js3 = json!("Herman Melville");
        let js4 = json!("J. R. R. Tolkien");
        let _ = test(
            template_json(),
            "$..author",
            vec![
                (&js1, "$['store']['book'][0]['author']"),
                (&js2, "$['store']['book'][1]['author']"),
                (&js3, "$['store']['book'][2]['author']"),
                (&js4, "$['store']['book'][3]['author']"),
            ],
        );
    }

    #[test]
    fn wildcard_test() {
        let js1 = json!("reference");
        let js2 = json!("fiction");
        let _ = test(
            template_json(),
            "$..book.[*].category",
            vec![
                (&js1, "$['store']['book'][0].['category']"),
                (&js2, "$['store']['book'][1].['category']"),
                (&js2, "$['store']['book'][2].['category']"),
                (&js2, "$['store']['book'][3].['category']"),
            ],
        );
        let js1 = json!("Nigel Rees");
        let js2 = json!("Evelyn Waugh");
        let js3 = json!("Herman Melville");
        let js4 = json!("J. R. R. Tolkien");
        let _ = test(
            template_json(),
            "$.store.book[*].author",
            vec![
                (&js1, "$['store']['book'][0]['author']"),
                (&js2, "$['store']['book'][1]['author']"),
                (&js3, "$['store']['book'][2]['author']"),
                (&js4, "$['store']['book'][3]['author']"),
            ],
        );
    }

    #[test]
    fn descendent_wildcard_test() {
        let js1 = json!("0-553-21311-3");
        let js2 = json!("0-395-19395-8");
        let _ = test(
            template_json(),
            "$..*.[?@].isbn",
            vec![
                (&js1, "$['store']['book'][2]['isbn']"),
                (&js2, "$['store']['book'][3]['isbn']"),
            ],
        );
    }

    #[test]
    fn field_test() {
        let value = json!({"active":1});
        let _ = test(
            r#"{"field":{"field":[{"active":1},{"passive":1}]}}"#,
            "$.field.field[?@.active]",
            vec![(&value, "$['field']['field'][0]")],
        );
    }

    #[test]
    fn index_index_test() {
        let value = json!("0-553-21311-3");
        let _ = test(
            template_json(),
            "$..book[2].isbn",
            vec![(&value, "$['store']['book'][2]['isbn']")],
        );
    }

    #[test]
    fn index_unit_index_test() {
        let value = json!("0-553-21311-3");
        let _ = test(
            template_json(),
            "$..book[2,4].isbn",
            vec![(&value, "$['store']['book'][2]['isbn']")],
        );
        let value1 = json!("0-395-19395-8");
        let _ = test(
            template_json(),
            "$..book[2,3].isbn",
            vec![
                (&value, "$['store']['book'][2]['isbn']"),
                (&value1, "$['store']['book'][3]['isbn']"),
            ],
        );
    }

    #[test]
    fn index_unit_keys_test() {
        let js1 = json!("Moby Dick");
        let js2 = json!(8.99);
        let js3 = json!("The Lord of the Rings");
        let js4 = json!(22.99);
        let _ = test(
            template_json(),
            "$..book[2,3]['title','price']",
            vec![
                (&js1, "$['store']['book'][2]['title']"),
                (&js3, "$['store']['book'][3]['title']"),
                (&js2, "$['store']['book'][2]['price']"),
                (&js4, "$['store']['book'][3]['price']"),
            ],
        );
    }

    #[test]
    fn index_slice_test() -> Parsed<()> {
        let i0 = "$['array'][0]";
        let i1 = "$['array'][1]";
        let i2 = "$['array'][2]";
        let i3 = "$['array'][3]";
        let i4 = "$['array'][4]";
        let i5 = "$['array'][5]";
        let i6 = "$['array'][6]";
        let i7 = "$['array'][7]";
        let i8 = "$['array'][8]";
        let i9 = "$['array'][9]";

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
        test(
            template_json(),
            "$.array[:]",
            vec![
                (&j0, i0),
                (&j1, i1),
                (&j2, i2),
                (&j3, i3),
                (&j4, i4),
                (&j5, i5),
                (&j6, i6),
                (&j7, i7),
                (&j8, i8),
                (&j9, i9),
            ],
        )?;
        test(
            template_json(),
            "$.array[1:4:2]",
            vec![(&j1, i1), (&j3, i3)],
        )?;
        test(
            template_json(),
            "$.array[::3]",
            vec![(&j0, i0), (&j3, i3), (&j6, i6), (&j9, i9)],
        )?;
        test(template_json(), "$.array[-1:]", vec![(&j9, i9)])?;
        test(template_json(), "$.array[-2:-1]", vec![(&j8, i8)])?;

        Ok(())
    }

    #[test]
    fn index_filter_test() -> Parsed<()> {
        let moby = json!("Moby Dick");
        let rings = json!("The Lord of the Rings");
        test(
            template_json(),
            "$..book[?@.isbn].title",
            vec![
                (&moby, "$['store']['book'][2]['title']"),
                (&rings, "$['store']['book'][3]['title']"),
            ],
        )?;
        let sword = json!("Sword of Honour");
        test(
            template_json(),
            "$..book[?(@.price != 8.95)].title",
            vec![
                (&sword, "$['store']['book'][1]['title']"),
                (&moby, "$['store']['book'][2]['title']"),
                (&rings, "$['store']['book'][3]['title']"),
            ],
        )?;
        let sayings = json!("Sayings of the Century");
        test(
            template_json(),
            "$..book[?(@.price == 8.95)].title",
            vec![(&sayings, "$['store']['book'][0]['title']")],
        )?;

        let js12 = json!(12.99);
        let js899 = json!(8.99);
        let js2299 = json!(22.99);
        test(
            template_json(),
            "$..book[?@.price >= 8.99].price",
            vec![
                (&js12, "$['store']['book'][1]['price']"),
                (&js899, "$['store']['book'][2]['price']"),
                (&js2299, "$['store']['book'][3]['price']"),
            ],
        )?;

        test(
            template_json(),
            "$..book[?(@.price >= $.expensive)].price",
            vec![
                (&js12, "$['store']['book'][1]['price']"),
                (&js2299, "$['store']['book'][3]['price']"),
            ],
        )?;
        Ok(())
    }

    #[test]
    fn union_quotes() -> Queried<()> {
        let json = json!({
          "a": "ab",
          "b": "bc"
        });

        let vec = js_path("$['a',\r'b']", &json)?;

        assert_eq!(
            vec,
            vec![
                (&json!("ab"), "$['a']".to_string()).into(),
                (&json!("bc"), "$['b']".to_string()).into(),
            ]
        );

        Ok(())
    }

    #[test]
    fn space_between_selectors() -> Queried<()> {
        let json = json!({
          "a": {
            "b": "ab"
          }
        });

        let vec = js_path("$['a'] \r['b']", &json)?;

        assert_eq!(vec, vec![(&json!("ab"), "$['a']['b']".to_string()).into(),]);

        Ok(())
    }
    #[test]
    fn space_in_search() -> Queried<()> {
        let json = json!(["foo", "123"]);

        let vec = js_path("$[?search(@\n,'[a-z]+')]", &json)?;

        assert_eq!(vec, vec![(&json!("foo"), "$[0]".to_string()).into(),]);

        Ok(())
    }
    #[test]
    fn filter_key() -> Queried<()> {
        let json = json!([
          {
            "a": "b",
            "d": "e"
          },
          {
            "a": 1,
            "d": "f"
          }
        ]);

        let vec = js_path("$[?@.a!=\"b\"]", &json)?;

        assert_eq!(
            vec,
            vec![(&json!({"a":1, "d":"f"}), "$[1]".to_string()).into(),]
        );

        Ok(())
    }

    #[test]
    fn regex_key() -> Queried<()> {
        let json = json!({
          "regex": "b.?b",
          "values": [
            "abc",
            "bcd",
            "bab",
            "bba",
            "bbab",
            "b",
            true,
            [],
            {}
          ]
        });

        let vec = js_path("$.values[?match(@, $.regex)]", &json)?;

        assert_eq!(
            vec,
            vec![(&json!("bab"), "$['values'][2]".to_string()).into(),]
        );

        Ok(())
    }
    #[test]
    fn name_sel() -> Queried<()> {
        let json = json!({
          "/": "A"
        });

        let vec = js_path("$['\\/']", &json)?;

        assert_eq!(vec, vec![(&json!("A"), "$['\\/']".to_string()).into(),]);

        Ok(())
    }
    #[test]
    fn unicode_fns() -> Queried<()> {
        let json = json!(["ж", "Ж", "1", "жЖ", true, [], {}]);

        let vec = js_path("$[?match(@, '\\\\p{Lu}')]", &json)?;

        assert_eq!(vec, vec![(&json!("Ж"), "$[1]".to_string()).into(),]);

        Ok(())
    }
    #[test]
    fn fn_res_can_not_compare() -> Queried<()> {
        let json = json!({});

        let vec = js_path("$[?match(@.a, 'a.*')==true]", &json);

        assert!(vec.is_err());

        Ok(())
    }
    #[test]
    fn too_small() -> Queried<()> {
        let json = json!({});

        let vec = js_path("$[-9007199254740992]", &json);

        assert!(vec.is_err());

        Ok(())
    }
    #[test]
    fn filter_data() -> Queried<()> {
        let json = json!({
          "a": 1,
          "b": 2,
          "c": 3
        });

        let vec: Vec<String> = json.query_only_path("$[?@<3]")?.into_iter().collect();

        assert_eq!(vec, vec!["$['a']".to_string(), "$['b']".to_string()]);

        Ok(())
    }
    #[test]
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
        assert_eq!(vec, vec![&json!({"a":100, "d":"e"})]);

        Ok(())
    }
    #[test]
    fn single_quote() -> Queried<()> {
        let json = json!({
          "a'": "A",
          "b": "B"
        });

        let vec = js_path("$[\"a'\"]", &json)?;
        assert_eq!(vec, vec![(&json!("A"), "$['\"a\'\"']".to_string()).into(),]);

        Ok(())
    }
    #[test]
    fn union() -> Queried<()> {
        let json = json!([0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

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

    #[test]
    fn basic_descendent() -> Queried<()> {
        let json = json!({
          "o": [
            0,
            1,
            [
              2,
              3
            ]
          ]
        });

        let vec = js_path("$..[1]", &json)?;
        assert_eq!(
            vec,
            vec![
                (&json!(1), "$['o'][1]".to_string()).into(),
                (&json!(3), "$['o'][2][1]".to_string()).into(),
            ]
        );

        Ok(())
    }
    #[test]
    fn filter_absent() -> Queried<()> {
        let json = json!([
          {
            "list": [
              1
            ]
          }
        ]);

        let vec = js_path("$[?@.absent==@.list[9]]", &json)?;
        assert_eq!(
            vec,
            vec![(&json!({"list": [1]}), "$[0]".to_string()).into(),]
        );

        Ok(())
    }

    #[test]
    fn filter_star() -> Queried<()> {
        let json = json!([1,[],[2],{},{"a": 3}]);

        let vec = json.query_with_path("$[?@.*]")?;
        assert_eq!(
            vec,
            vec![
                (&json!([2]), "$[2]".to_string()).into(),
                (&json!({"a": 3}), "$[4]".to_string()).into(),
            ]
        );

        Ok(())
    }

    #[test]
    fn space_test() -> Queried<()> {
        let json = json!({ " ": "A"});

        let vec = json.query_with_path("$[' ']")?;
        assert_eq!(vec, vec![(&json!("A"), "$[\' \']".to_string()).into(),]);

        Ok(())
    }
    #[test]
    fn neg_idx() -> Queried<()> {
        let json = json!(["first", "second"]);

        let vec = json.query_with_path("$[-2]")?;
        assert_eq!(vec, vec![(&json!("first"), "$[0]".to_string()).into(),]);

        Ok(())
    }

    #[test]
    fn filter_slice() -> Queried<()> {
        let json = json!([
          1,
          [],
          [
            2
          ],
          [
            2,
            3,
            4
          ],
          {},
          {
            "a": 3
          }
        ]);

        let vec = json.query_with_path("$[?@[0:2]]")?;
        assert_eq!(
            vec,
            vec![
                (&json!([2]), "$[2]").into(),
                (&json!([2, 3, 4]), "$[3]").into(),
            ]
        );

        Ok(())
    }

    #[test]
    fn surr_pairs() -> Queried<()> {
        let json = json!({
          "𝄞": "A"
        });
        let vec = json.query_with_path("$['𝄞']")?;
        assert_eq!(vec, vec![(&json!("A"), "$['𝄞']".to_string()).into()]);

        Ok(())
    }
    #[test]
    fn tab_key() -> Queried<()> {
        let json = json!({
          "\\t": "A"
        });
        let vec = json.query_with_path("$['\\t']")?;
        assert_eq!(vec, vec![(&json!("A"), "$['\\t']".to_string()).into()]);

        Ok(())
    }
    #[test]
    fn escaped_up_hex() -> Queried<()> {
        let json = json!({
          "☺": "A"
        });
        let vec = json.query_with_path("$['☺']")?;
        assert_eq!(vec, vec![(&json!("A"), "$['☺']".to_string()).into()]);

        Ok(())
    }
    #[test]
    fn carr_return() -> Queried<()> {
        let json = json!({
          "\\r": "A"
        });
        let vec = json.query_with_path("$['\\r']")?;
        assert_eq!(vec, vec![(&json!("A"), "$['\\r']".to_string()).into()]);

        Ok(())
    }
}
