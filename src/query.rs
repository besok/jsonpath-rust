pub mod queryable;
mod segment;
mod selector;
mod test_function;
mod jp_query;

use crate::path::JsonLike;
use crate::query::queryable::Queryable;
use crate::JsonPathParserError;

type QueryPath = String;
type Queried<T> = Result<T, JsonPathParserError>;

#[derive(Debug, Clone, PartialEq)]
pub struct Data<'a, T: Queryable> {
    pub pointer: &'a T,
    pub path: QueryPath,
}

impl<'a, T: Queryable> Data<'a, T> {
    pub fn new(pointer: &'a T, path: QueryPath) -> Self {
        Data { pointer, path }
    }

    pub fn key(pointer: &'a T, path: QueryPath, key: &str) -> Self {
        Data {
            pointer,
            path: format!("{}.['{}']", path, key),
        }
    }
    pub fn idx(pointer: &'a T, path: QueryPath, index: usize) -> Self {
        Data {
            pointer,
            path: format!("{}[{}]", path, index),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Step<'a, T: Queryable> {
    Ref(Data<'a, T>),
    Refs(Vec<Data<'a, T>>),
    Value(T),
    Nothing,
}

impl<'a, T: Queryable> Default for Step<'a, T> {
    fn default() -> Self {
        Step::Nothing
    }
}

impl<'a, T: Queryable> Step<'a, T> {
    pub fn reduce(self, other: Step<'a, T>) -> Step<'a, T> {
        match (self, other) {
            (Step::Ref(data), Step::Ref(data2)) => Step::Refs(vec![data, data2]),
            (Step::Ref(data), Step::Refs(data_vec)) => {
                Step::Refs(data_vec.into_iter().chain(vec![data]).collect())
            }
            (Step::Refs(data_vec), Step::Ref(data)) => {
                Step::Refs(data_vec.into_iter().chain(vec![data]).collect())
            }
            (Step::Refs(data_vec), Step::Refs(data_vec2)) => {
                Step::Refs(data_vec.into_iter().chain(data_vec2).collect())
            }
            _ => Step::Nothing,
        }
    }

    pub fn flat_map<F>(self, f: F) -> Step<'a, T>
    where
        F: Fn(Data<'a, T>) -> Step<'a, T>,
    {
        match self {
            Step::Ref(data) => f(data),
            Step::Refs(data_vec) => Step::Refs(
                data_vec
                    .into_iter()
                    .flat_map(|data| match f(data) {
                        Step::Ref(data) => vec![data],
                        Step::Refs(data_vec) => data_vec,
                        _ => vec![],
                    })
                    .collect::<Vec<_>>(),
            ),
            _ => Step::Nothing,
        }
    }

    pub fn ok(self) -> Option<Vec<Data<'a, T>>> {
        match self {
            Step::Ref(data) => Some(vec![data]),
            Step::Refs(data) => Some(data),
            _ => None,
        }
    }

    pub fn new_ref(data: Data<'a, T>) -> Step<'a, T> {
        Step::Ref(data)
    }

    pub fn new_refs(data: Vec<Data<'a, T>>) -> Step<'a, T> {
        Step::Refs(data)
    }
}

pub trait Query {
    fn process<'a, T: Queryable>(&self, step: Step<'a, T>) -> Step<'a, T>;
}
