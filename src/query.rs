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

pub type QueryPath = String;
pub type Queried<T> = Result<T, JsonPathError>;

pub trait Query {
    fn process<'a, T: Queryable>(&self, state: State<'a, T>) -> State<'a, T>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum QueryResult<'a, T: Queryable> {
    Val(T),
    Ref(&'a T, QueryPath),
}

impl<'a, T: Queryable> From<(&'a T, QueryPath)> for QueryResult<'a, T> {
    fn from((inner, path): (&'a T, QueryPath)) -> Self {
        QueryResult::Ref(inner, path)
    }
}

impl<'a, T: Queryable> QueryResult<'a, T> {
    pub fn val(self) -> Cow<'a, T> {
        match self {
            QueryResult::Val(v) => Cow::Owned(v),
            QueryResult::Ref(v, _) => Cow::Borrowed(v),
        }
    }
    pub fn path(self) -> Option<QueryPath> {
        match self {
            QueryResult::Val(_) => None,
            QueryResult::Ref(_, path) => Some(path),
        }
    }
}

impl<T: Queryable> From<T> for QueryResult<'_, T> {
    fn from(value: T) -> Self {
        QueryResult::Val(value)
    }
}
impl<'a, T: Queryable> From<Pointer<'a, T>> for QueryResult<'a, T> {
    fn from(pointer: Pointer<'a, T>) -> Self {
        QueryResult::Ref(pointer.inner, pointer.path)
    }
}

pub fn js_path<'a, T: Queryable>(path: &str, value: &'a T) -> Queried<Vec<QueryResult<'a, T>>> {
    match parse_json_path(path)?.process(State::root(value)).data {
        Data::Ref(p) => Ok(vec![p.into()]),
        Data::Refs(refs) => Ok(refs.into_iter().map(Into::into).collect()),
        Data::Value(v) => Ok(vec![v.into()]),
        Data::Nothing => Ok(vec![]),
    }
}

pub fn js_path_vals<'a, T: Queryable>(path: &str, value: &'a T) -> Queried<Vec<Cow<'a, T>>> {
    Ok(js_path(path, value)?
        .into_iter()
        .map(|r| r.val())
        .collect::<Vec<_>>())
}
pub fn js_path_path<T: Queryable>(path: &str, value: &T) -> Queried<Vec<Option<String>>> {
    Ok(js_path(path, value)?
        .into_iter()
        .map(|r| r.path())
        .collect::<Vec<_>>())
}

pub trait JsonPath: Queryable {
    fn query_with_path(&self, path: &str) -> Queried<Vec<QueryResult<Self>>> {
        js_path(path, self)
    }
    fn query_only_path(&self, path: &str) -> Queried<Vec<Option<String>>> {
        js_path_path(path, self)
    }
    fn query(&self, path: &str) -> Queried<Vec<Cow<Self>>> {
        js_path_vals(path, self)
    }
}

impl JsonPath for Value {}

fn x(v: &'static Value) -> Queried<Vec<Cow<'static, Value>>> {
    v.query("a")
}
