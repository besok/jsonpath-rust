use crate::parser::errors::JsonPathError;
use crate::parser::model::{JpQuery, Segment, Selector};
use crate::parser::{parse_json_path, Parsed};
use crate::query::QueryPath;
use serde_json::Value;
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
    fn reference<T>(&self, path: T) -> Option<&Self>
    where
        T: Into<QueryPath>,
    {
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
    ///
    /// ```
    fn reference_mut<T>(&mut self, path: T) -> Option<&mut Self>
    where
        T: Into<QueryPath>,
    {
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

    /// Custom extension function for JSONPath queries.
    ///
    /// This function allows for custom operations to be performed on JSON data
    /// based on the provided `name` and `args`.
    ///
    /// # Arguments
    ///
    /// * `name` - A string slice that holds the name of the custom function.
    /// * `args` - A vector of `Cow<Self>` that holds the arguments for the custom function.
    ///
    /// # Returns
    ///
    /// Returns a `Self` value which is the result of the custom function. If the function
    /// name is not recognized, it returns `Self::null()`.
    ///
    /// # Custom Functions
    ///
    /// * `"in"` - Checks if the first argument is in the array provided as the second argument.
    ///   Example: `$.elems[?in(@, $.list)]` - Returns elements from $.elems that are present in $.list
    ///
    /// * `"nin"` - Checks if the first argument is not in the array provided as the second argument.
    ///   Example: `$.elems[?nin(@, $.list)]` - Returns elements from $.elems that are not present in $.list
    ///
    /// * `"none_of"` - Checks if none of the elements in the first array are in the second array.
    ///   Example: `$.elems[?none_of(@, $.list)]` - Returns arrays from $.elems that have no elements in common with $.list
    ///
    /// * `"any_of"` - Checks if any of the elements in the first array are in the second array.
    ///   Example: `$.elems[?any_of(@, $.list)]` - Returns arrays from $.elems that have at least one element in common with $.list
    ///
    /// * `"subset_of"` - Checks if all elements in the first array are in the second array.
    ///   Example: `$.elems[?subset_of(@, $.list)]` - Returns arrays from $.elems where all elements are present in $.list
    fn extension_custom(name: &str, args: Vec<Cow<Self>>) -> Self {
        match name {
            "in" => match args.as_slice() {
                [lhs, rhs] => match rhs.as_array() {
                    Some(elements) => elements.iter().any(|item| item == lhs.as_ref()).into(),
                    None => Self::null(),
                },
                _ => Self::null(),
            },
            "nin" => match args.as_slice() {
                [lhs, rhs] => match rhs.as_array() {
                    Some(elements) => (!elements.iter().any(|item| item == lhs.as_ref())).into(),
                    None => Self::null(),
                },
                _ => Self::null(),
            },
            "none_of" => match args.as_slice() {
                [lhs, rhs] => match (lhs.as_array(), rhs.as_array()) {
                    (Some(lhs_arr), Some(rhs_arr)) => lhs_arr
                        .iter()
                        .all(|lhs| !rhs_arr.iter().any(|rhs| lhs == rhs))
                        .into(),
                    _ => Self::null(),
                },
                _ => Self::null(),
            },
            "any_of" => match args.as_slice() {
                [lhs, rhs] => match (lhs.as_array(), rhs.as_array()) {
                    (Some(lhs_arr), Some(rhs_arr)) => lhs_arr
                        .iter()
                        .any(|lhs| rhs_arr.iter().any(|rhs| lhs == rhs))
                        .into(),
                    _ => Self::null(),
                },
                _ => Self::null(),
            },
            "subset_of" => match args.as_slice() {
                [lhs, rhs] => match (lhs.as_array(), rhs.as_array()) {
                    (Some(lhs_arr), Some(rhs_arr)) => lhs_arr
                        .iter()
                        .all(|lhs| rhs_arr.iter().any(|rhs| lhs == rhs))
                        .into(),
                    _ => Self::null(),
                },
                _ => Self::null(),
            },
            _ => Self::null(),
        }
    }

    fn reference<T>(&self, path: T) -> Option<&Self>
    where
        T: Into<QueryPath>,
    {
        convert_js_path(&path.into())
            .ok()
            .and_then(|p| self.pointer(p.as_str()))
    }

    fn reference_mut<T>(&mut self, path: T) -> Option<&mut Self>
    where
        T: Into<QueryPath>,
    {
        convert_js_path(&path.into())
            .ok()
            .and_then(|p| self.pointer_mut(p.as_str()))
    }
}

fn convert_js_path(path: &str) -> Parsed<String> {
    let JpQuery { segments } = parse_json_path(path)?;

    let mut path = String::new();
    for segment in segments {
        match segment {
            Segment::Selector(Selector::Name(name)) => {
                path.push_str(&format!("/{}", name));
            }
            Segment::Selector(Selector::Index(index)) => {
                path.push_str(&format!("/{}", index));
            }
            s => {
                return Err(JsonPathError::InvalidJsonPath(format!(
                    "Invalid segment: {:?}",
                    s
                )));
            }
        }
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use crate::query::Queried;
    use serde_json::json;
    use std::borrow::Cow;
    use crate::JsonPath;
    use crate::parser::Parsed;
    use crate::query::queryable::{convert_js_path, Queryable};

    #[test]
    fn in_smoke() -> Queried<()> {
        let json = json!({
            "elems": ["test", "t1", "t2"],
            "list": ["test", "test2", "test3"],
        });

        let res = json.query("$.elems[?in(@, $.list)]")?;

        assert_eq!(res, [Cow::Borrowed(&json!("test"))]);

        Ok(())
    }
    #[test]
    fn nin_smoke() -> Queried<()> {
        let json = json!({
            "elems": ["test", "t1", "t2"],
            "list": ["test", "test2", "test3"],
        });

        let res = json.query("$.elems[?nin(@, $.list)]")?;

        assert_eq!(
            res,
            [Cow::Borrowed(&json!("t1")), Cow::Borrowed(&json!("t2"))]
        );

        Ok(())
    }
    #[test]
    fn none_of_smoke() -> Queried<()> {
        let json = json!({
            "elems": [  ["t1", "_"], ["t2", "t5"], ["t4"]],
            "list": ["t1","t2", "t3"],
        });

        let res = json.query("$.elems[?none_of(@, $.list)]")?;

        assert_eq!(res, [Cow::Borrowed(&json!(["t4"]))]);

        Ok(())
    }
    #[test]
    fn any_of_smoke() -> Queried<()> {
        let json = json!({
            "elems": [  ["t1", "_"], ["t4", "t5"], ["t4"]],
            "list": ["t1","t2", "t3"],
        });

        let res = json.query("$.elems[?any_of(@, $.list)]")?;

        assert_eq!(res, [Cow::Borrowed(&json!(["t1", "_"]))]);

        Ok(())
    }
    #[test]
    fn subset_of_smoke() -> Queried<()> {
        let json = json!({
            "elems": [  ["t1", "t2"], ["t4", "t5"], ["t6"]],
            "list": ["t1","t2", "t3"],
        });

        let res = json.query("$.elems[?subset_of(@, $.list)]")?;

        assert_eq!(res, [Cow::Borrowed(&json!(["t1", "t2"]))]);

        Ok(())
    }


    #[test]
    fn convert_paths() -> Parsed<()> {
        let r = convert_js_path("$.a.b[2]")?;
        assert_eq!(r, "/a/b/2");

        Ok(())
    }

    #[test]
    fn test_references() -> Parsed<()> {
        let mut json = json!({
            "a": {
                "b": {
                    "c": 42
                }
            }
        });

        let r = convert_js_path("$.a.b.c")?;

        if let Some(v) = json.pointer_mut(r.as_str()) {
            *v = json!(43);
        }

        assert_eq!(
            json,
            json!({
                "a": {
                    "b": {
                        "c": 43
                    }
                }
            })
        );

        Ok(())
    }
    #[test]
    fn test_js_reference() ->Parsed<()> {
        let mut json = json!({
            "a": {
                "b": {
                    "c": 42
                }
            }
        });

        if let Some(v) = json.reference_mut("$.a.b.c") {
            *v = json!(43);
        }

        assert_eq!(
            json,
            json!({
                "a": {
                    "b": {
                        "c": 43
                    }
                }
            })
        );

        Ok(())
    }
}
