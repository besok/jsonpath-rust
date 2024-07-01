use crate::JsonPathValue;
use serde_json::Value;

use crate::parser::model::{Function, JsonPath, JsonPathIndex, Operand};
use crate::path::index::{ArrayIndex, ArraySlice, Current, FilterPath, UnionIndex};
use crate::path::top::*;

/// The module is in charge of processing [[JsonPathIndex]] elements
mod index;
/// The module is a helper module providing the set of helping funcitons to process a json elements
mod json;
/// The module is responsible for processing of the [[JsonPath]] elements
mod top;

/// The trait defining the behaviour of processing every separated element.
/// type Data usually stands for json [[Value]]
/// The trait also requires to have a root json to process.
/// It needs in case if in the filter there will be a pointer to the absolute path
pub trait Path<'a> {
    type Data;
    /// when every element needs to handle independently
    fn find(&self, input: JsonPathValue<'a, Self::Data>) -> Vec<JsonPathValue<'a, Self::Data>> {
        vec![input]
    }
    /// when the whole output needs to handle
    fn flat_find(
        &self,
        input: Vec<JsonPathValue<'a, Self::Data>>,
        _is_search_length: bool,
    ) -> Vec<JsonPathValue<'a, Self::Data>> {
        input.into_iter().flat_map(|d| self.find(d)).collect()
    }
    /// defines when we need to invoke `find` or `flat_find`
    fn needs_all(&self) -> bool {
        false
    }
}

/// all known Paths, mostly to avoid a dynamic Box and vtable for internal function
pub(crate) enum TopPaths<'a> {
    RootPointer(RootPointer<'a, Value>),
    ObjectField(ObjectField<'a>),
    Chain(Chain<'a>),
    Wildcard(Wildcard),
    DescentObject(DescentObject<'a>),
    DescentWildcard(DescentWildcard),
    Current(Current<'a>),
    ArrayIndex(ArrayIndex),
    ArraySlice(ArraySlice),
    UnionIndex(UnionIndex<'a>),
    FilterPath(FilterPath<'a>),
    IdentityPath(IdentityPath),
    FnPath(FnPath),
}

impl<'a> Path<'a> for TopPaths<'a> {
    type Data = Value;

    fn find(&self, input: JsonPathValue<'a, Self::Data>) -> Vec<JsonPathValue<'a, Self::Data>> {
        match self {
            TopPaths::RootPointer(inner) => inner.find(input),
            TopPaths::ObjectField(inner) => inner.find(input),
            TopPaths::Chain(inner) => inner.find(input),
            TopPaths::Wildcard(inner) => inner.find(input),
            TopPaths::DescentObject(inner) => inner.find(input),
            TopPaths::DescentWildcard(inner) => inner.find(input),
            TopPaths::Current(inner) => inner.find(input),
            TopPaths::ArrayIndex(inner) => inner.find(input),
            TopPaths::ArraySlice(inner) => inner.find(input),
            TopPaths::UnionIndex(inner) => inner.find(input),
            TopPaths::FilterPath(inner) => inner.find(input),
            TopPaths::IdentityPath(inner) => inner.find(input),
            TopPaths::FnPath(inner) => inner.find(input),
        }
    }

    fn flat_find(
        &self,
        input: Vec<JsonPathValue<'a, Self::Data>>,
        _is_search_length: bool,
    ) -> Vec<JsonPathValue<'a, Self::Data>> {
        match self {
            TopPaths::RootPointer(inner) => inner.flat_find(input, _is_search_length),
            TopPaths::ObjectField(inner) => inner.flat_find(input, _is_search_length),
            TopPaths::Chain(inner) => inner.flat_find(input, _is_search_length),
            TopPaths::Wildcard(inner) => inner.flat_find(input, _is_search_length),
            TopPaths::DescentObject(inner) => inner.flat_find(input, _is_search_length),
            TopPaths::DescentWildcard(inner) => inner.flat_find(input, _is_search_length),
            TopPaths::Current(inner) => inner.flat_find(input, _is_search_length),
            TopPaths::ArrayIndex(inner) => inner.flat_find(input, _is_search_length),
            TopPaths::ArraySlice(inner) => inner.flat_find(input, _is_search_length),
            TopPaths::UnionIndex(inner) => inner.flat_find(input, _is_search_length),
            TopPaths::FilterPath(inner) => inner.flat_find(input, _is_search_length),
            TopPaths::IdentityPath(inner) => inner.flat_find(input, _is_search_length),
            TopPaths::FnPath(inner) => inner.flat_find(input, _is_search_length),
        }
    }

    fn needs_all(&self) -> bool {
        match self {
            TopPaths::RootPointer(inner) => inner.needs_all(),
            TopPaths::ObjectField(inner) => inner.needs_all(),
            TopPaths::Chain(inner) => inner.needs_all(),
            TopPaths::Wildcard(inner) => inner.needs_all(),
            TopPaths::DescentObject(inner) => inner.needs_all(),
            TopPaths::DescentWildcard(inner) => inner.needs_all(),
            TopPaths::Current(inner) => inner.needs_all(),
            TopPaths::ArrayIndex(inner) => inner.needs_all(),
            TopPaths::ArraySlice(inner) => inner.needs_all(),
            TopPaths::UnionIndex(inner) => inner.needs_all(),
            TopPaths::FilterPath(inner) => inner.needs_all(),
            TopPaths::IdentityPath(inner) => inner.needs_all(),
            TopPaths::FnPath(inner) => inner.needs_all(),
        }
    }
}

/// The basic type for instances.
pub(crate) type PathInstance<'a> = Box<dyn Path<'a, Data = Value> + 'a>;

/// The major method to process the top part of json part
pub(crate) fn json_path_instance<'a>(json_path: &'a JsonPath, root: &'a Value) -> TopPaths<'a> {
    match json_path {
        JsonPath::Root => TopPaths::RootPointer(RootPointer::new(root)),
        JsonPath::Field(key) => TopPaths::ObjectField(ObjectField::new(key)),
        JsonPath::Chain(chain) => TopPaths::Chain(Chain::from(chain, root)),
        JsonPath::Wildcard => TopPaths::Wildcard(Wildcard {}),
        JsonPath::Descent(key) => TopPaths::DescentObject(DescentObject::new(key)),
        JsonPath::DescentW => TopPaths::DescentWildcard(DescentWildcard),
        JsonPath::Current(value) => TopPaths::Current(Current::from(value, root)),
        JsonPath::Index(JsonPathIndex::Single(index)) => {
            TopPaths::ArrayIndex(ArrayIndex::new(index.as_u64().unwrap() as usize))
        }
        JsonPath::Index(JsonPathIndex::Slice(s, e, step)) => {
            TopPaths::ArraySlice(ArraySlice::new(*s, *e, *step))
        }
        JsonPath::Index(JsonPathIndex::UnionKeys(elems)) => {
            TopPaths::UnionIndex(UnionIndex::from_keys(elems))
        }
        JsonPath::Index(JsonPathIndex::UnionIndex(elems)) => {
            TopPaths::UnionIndex(UnionIndex::from_indexes(elems))
        }
        JsonPath::Index(JsonPathIndex::Filter(fe)) => {
            TopPaths::FilterPath(FilterPath::new(fe, root))
        }
        JsonPath::Empty => TopPaths::IdentityPath(IdentityPath {}),
        JsonPath::Fn(Function::Length) => TopPaths::FnPath(FnPath::Size),
    }
}

/// The method processes the operand inside the filter expressions
fn process_operand<'a>(op: &'a Operand, root: &'a Value) -> PathInstance<'a> {
    Box::new(match op {
        Operand::Static(v) => json_path_instance(&JsonPath::Root, v),
        Operand::Dynamic(jp) => json_path_instance(jp, root),
    })
}
