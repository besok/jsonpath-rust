mod atom;
mod comparable;
mod comparison;
mod filter;
mod jp_query;
pub mod queryable;
mod segment;
mod selector;
mod state;
mod test;
mod test_function;

use crate::parser2::errors2::JsonPathError;
use crate::parser2::parse_json_path;
use crate::path::JsonLike;
use crate::query::queryable::Queryable;
use crate::query::state::{Data, Pointer};
use serde_json::Value;
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
pub enum QueryRes<'a, T: Queryable> {
    Val(T),
    Ref(&'a T, QueryPath),
}

impl<'a, T: Queryable> From<(&'a T, QueryPath)> for QueryRes<'a, T> {
    fn from((inner, path): (&'a T, QueryPath)) -> Self {
        QueryRes::Ref(inner, path)
    }
}

impl<'a, T: Queryable> QueryRes<'a, T> {
    pub fn val(self) -> Cow<'a, T> {
        match self {
            QueryRes::Val(v) => Cow::Owned(v),
            QueryRes::Ref(v, _) => Cow::Borrowed(v),
        }
    }
    pub fn path(self) -> Option<QueryPath> {
        match self {
            QueryRes::Val(_) => None,
            QueryRes::Ref(_, path) => Some(path),
        }
    }
}

impl<T: Queryable> From<T> for QueryRes<'_, T> {
    fn from(value: T) -> Self {
        QueryRes::Val(value)
    }
}
impl<'a, T: Queryable> From<Pointer<'a, T>> for QueryRes<'a, T> {
    fn from(pointer: Pointer<'a, T>) -> Self {
        QueryRes::Ref(pointer.inner, pointer.path)
    }
}

/// The main function to process a JSONPath query.
/// It takes a path and a value, and returns a vector of `QueryResult` thus values + paths.
pub fn js_path<'a, T: Queryable>(path: &str, value: &'a T) -> Queried<Vec<QueryRes<'a, T>>> {
    match parse_json_path(path)?.process(State::root(value)).data {
        Data::Ref(p) => Ok(vec![p.into()]),
        Data::Refs(refs) => Ok(refs.into_iter().map(Into::into).collect()),
        Data::Value(v) => Ok(vec![v.into()]),
        Data::Nothing => Ok(vec![]),
    }
}

/// A convenience function to process a JSONPath query and return a vector of values, omitting the path.
pub fn js_path_vals<'a, T: Queryable>(path: &str, value: &'a T) -> Queried<Vec<Cow<'a, T>>> {
    Ok(js_path(path, value)?
        .into_iter()
        .map(|r| r.val())
        .collect::<Vec<_>>())
}

/// A convenience function to process a JSONPath query and return a vector of paths, omitting the values.
pub fn js_path_path<T: Queryable>(path: &str, value: &T) -> Queried<Vec<Option<String>>> {
    Ok(js_path(path, value)?
        .into_iter()
        .map(|r| r.path())
        .collect::<Vec<_>>())
}

/// A trait for types that can be queried with JSONPath.
pub trait JsonPath: Queryable {
    /// Queries the value with a JSONPath expression and returns a vector of `QueryResult`.
    fn query_with_path(&self, path: &str) -> Queried<Vec<QueryRes<Self>>> {
        js_path(path, self)
    }

    /// Queries the value with a JSONPath expression and returns a vector of values.
    fn query_only_path(&self, path: &str) -> Queried<Vec<Option<String>>> {
        js_path_path(path, self)
    }

    /// Queries the value with a JSONPath expression and returns a vector of values, omitting the path.
    fn query(&self, path: &str) -> Queried<Vec<Cow<Self>>> {
        js_path_vals(path, self)
    }
}

impl JsonPath for Value {}

#[cfg(test)]
mod tests {
    use crate::query::queryable::Queryable;
    use crate::query::{JsonPath, Queried};
    use serde_json::json;

    fn update_by_path_test() -> Queried<()> {
        let mut json = json!([
            {"verb": "RUN","distance":[1]},
            {"verb": "TEST"},
            {"verb": "DO NOT RUN"}
        ]);

        let path = json.query_only_path("$.[?(@.verb == 'RUN')]")?;
        let elem = path.first().cloned().flatten().unwrap_or_default();

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
}
