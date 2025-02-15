pub mod queryable;
mod test_function;
mod segment;
mod selector;

use crate::JsonPathParserError;
use crate::path::JsonLike;
use crate::query::queryable::Queryable;

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

    pub fn new_key(pointer: &'a T, path:QueryPath, key: &str) -> Self {
        Data { pointer, path: format!("{}.['{}']", path, key) }
    }
    pub fn new_idx(pointer: &'a T, path:QueryPath, index:usize) -> Self {
        Data { pointer, path: format!("{}[{}]", path, index) }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Step<'a, T: Queryable> {
    Data(Data<'a, T>),
    NewData(T),
    Nothing,
}

impl<'a, T: Queryable> Default for Step<'a, T> {
    fn default() -> Self {
        Step::Nothing
    }
}

impl<'a, T: Queryable> Step<'a, T> {

    pub fn flat_map<F>(self, f: F) -> Step<'a, T>
    where
        F: FnOnce(Data<'a, T>) -> Step<'a, T>,
    {
        match self {
            Step::Data(data) => f(data),
            _ => Step::Nothing,
        }
    }

    pub fn ok(self) -> Option<Data<'a, T>> {
        match self {
            Step::Data(data) => Some(data),
            _ => None,
        }
    }

    pub fn new_ref(data: Data<'a,T>) -> Step<'a, T> {
        Step::Data(data)
    }


}

pub trait Query {
    fn process<'a, T: Queryable>(&self, step: Step<'a, T>) -> Step<'a, T>;
}
