use serde_json::Value;

use crate::parser::model::{JsonPath, JsonPathIndex, Operand};
use crate::path::index::{ArrayIndex, ArraySlice, Current, Filter, UnionIndex};
use crate::path::top::*;

mod top;
mod index;
mod json;

pub trait Path<'a> {
    type Data;
    fn find(&self, data: &'a Self::Data) -> Vec<&'a Self::Data>;
}

pub type PathInstance<'a> = Box<dyn Path<'a, Data=Value> + 'a>;



pub fn json_path_instance<'a>(json_path: &'a JsonPath, root: &'a Value) -> PathInstance<'a> {
    match json_path {
        JsonPath::Root => Box::new(RootPointer::new(root)),
        JsonPath::Field(key) => Box::new(ObjectField::new(key)),
        JsonPath::Chain(chain) => Box::new(Chain::from(chain, root)),
        JsonPath::Wildcard => Box::new(Wildcard {}),
        JsonPath::Descent(key) => Box::new(DescentObjectField::new(key)),
        JsonPath::Current(value) => Box::new(Current::from(&**value,root)),
        JsonPath::Index(index) => process_index(index, root),
        JsonPath::Empty => Box::new(IdentityPath {})
    }
}

fn process_index<'a>(json_path_index: &'a JsonPathIndex, root: &'a Value) -> PathInstance<'a> {
    match json_path_index {
        JsonPathIndex::Single(index) => Box::new(ArrayIndex::new(*index)),
        JsonPathIndex::Slice(s, e, step) => Box::new(ArraySlice::new(*s, *e, *step)),
        JsonPathIndex::UnionKeys(elems) => Box::new(UnionIndex::from_keys(elems)),
        JsonPathIndex::UnionIndex(elems) => Box::new(UnionIndex::from_indexes(elems)),
        JsonPathIndex::Filter(l, op, r) => Box::new(Filter::new(l, r, op, root)),
    }
}

fn process_operand<'a>(op: &'a Operand, root: &'a Value) -> PathInstance<'a> {
    match op {
        Operand::Static(v) => json_path_instance(&JsonPath::Root, v),
        Operand::Dynamic(jp) => json_path_instance(jp, root)
    }
}
