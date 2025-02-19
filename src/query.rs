mod comparison;
mod jp_query;
pub mod queryable;
mod segment;
mod selector;
mod test_function;
mod state;
mod comparable;
mod test;
mod filter;
mod atom;

use state::State;
use crate::path::JsonLike;
use crate::query::queryable::Queryable;
use crate::JsonPathParserError;

type QueryPath = String;
type Queried<T> = Result<T, JsonPathParserError>;


pub trait Query {
    fn process<'a, T: Queryable>(&self, state: State<'a, T>) -> State<'a, T>;
}
