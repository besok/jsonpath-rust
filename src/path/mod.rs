//! The basic module denotes the strategy of processing jsonpath.
//! Overall, the escape sequence is the following one:
//! - define the json root
//! - define the json path structure from the parsing [[JsonPath]]
//! - transform json path into the [[PathInstance]]
//!
use std::borrow::Borrow;
use serde_json::{Value};

use crate::parser::model::{JsonPath, JsonPathIndex, Operand};
use crate::path::index::{ArrayIndex, ArraySlice, Current, FilterPath, UnionIndex};
use crate::path::top::*;

/// The module is responsible for processing of the [[JsonPath]] elements
mod top;
/// The module is in charge of processing [[JsonPathIndex]] elements
mod index;
/// The module is a helper module providing the set of helping funcitons to process a json elements
mod json;

pub enum PathData<'a, Data> {
    Ref(&'a Data),
    Slice(Data),
}

impl<'a, Data> PathData<'a, Data> {
    fn map_ref<F>(self, mapper: F) -> Vec<PathData<'a, Data>>
        where F: FnOnce(&'a Data) -> Vec<&'a Data>
    {
        match self {
            PathData::Ref(r) => {
                mapper(r)
                    .into_iter()
                    .map(PathData::Ref)
                    .collect()
            }
            PathData::Slice(_) => vec![]
        }
    }
}

pub trait Path2<'a> {
    type Data;
    fn find2(&self, data: PathData<'a, Self::Data>) -> Vec<PathData<'a, Self::Data>>;
}


/// The trait defining the behaviour of processing every separated element.
/// type Data usually stands for json [[Value]]
/// The trait also requires to have a root json to process.
/// It needs in case if in the filter there will be a pointer to the absolute path
pub trait Path<'a> {
    type Data;
    fn find(&self, data: &'a Self::Data) -> Vec<&'a Self::Data>;
}

/// The basic type for instances.
pub type PathInstance<'a> = Box<dyn Path<'a, Data=Value> + 'a>;

/// The major method to process the top part of json part
pub fn json_path_instance<'a>(json_path: &'a JsonPath, root: &'a Value) -> PathInstance<'a> {
    match json_path {
        JsonPath::Root => Box::new(RootPointer::new(root)),
        JsonPath::Field(key) => Box::new(ObjectField::new(key)),
        JsonPath::Chain(chain) => Box::new(Chain::from(chain, root)),
        JsonPath::Wildcard => Box::new(Wildcard {}),
        JsonPath::Descent(key) => Box::new(DescentObjectField::new(key)),
        JsonPath::Current(value) => Box::new(Current::from(&**value, root)),
        JsonPath::Index(index) => process_index(index, root),
        JsonPath::Empty => Box::new(IdentityPath {})
    }
}

/// The method processes the indexes(all expressions indie [])
fn process_index<'a>(json_path_index: &'a JsonPathIndex, root: &'a Value) -> PathInstance<'a> {
    match json_path_index {
        JsonPathIndex::Single(index) => Box::new(ArrayIndex::new(index.as_u64().unwrap() as usize)),
        JsonPathIndex::Slice(s, e, step) => Box::new(ArraySlice::new(*s, *e, *step)),
        JsonPathIndex::UnionKeys(elems) => Box::new(UnionIndex::from_keys(elems)),
        JsonPathIndex::UnionIndex(elems) => Box::new(UnionIndex::from_indexes(elems)),
        JsonPathIndex::Filter(fe) => Box::new(FilterPath::new(fe, root)),
    }
}

/// The method processes the operand inside the filter expressions
fn process_operand<'a>(op: &'a Operand, root: &'a Value) -> PathInstance<'a> {
    match op {
        Operand::Static(v) => json_path_instance(&JsonPath::Root, v),
        Operand::Dynamic(jp) => json_path_instance(jp, root)
    }
}
