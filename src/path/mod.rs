use serde_json::Value;

use crate::parser::model::{JsonPath, JsonPathIndex, Operand};
use crate::path::top::*;
use crate::path::index::{Current, ArrayIndex, ArraySlice, UnionIndex, Filter};

mod top;
mod index;

pub(crate) trait Path<'a> {
    type Data;
    fn path(&self, data: &'a Self::Data) -> Vec<&'a Self::Data>;
}

type PathInstance<'a> = Box<dyn Path<'a, Data=Value> + 'a>;

fn process_path<'a>(json_path: &'a JsonPath, root: &'a Value) -> PathInstance<'a> {
    match json_path {
        JsonPath::Root => Box::new(RootPointer::new(root)),
        JsonPath::Field(key) => Box::new(ObjectField::new(key)),
        JsonPath::Chain(chain) => Box::new(Chain::from(chain, root)),
        JsonPath::Wildcard => Box::new(Wildcard {}),
        JsonPath::Descent(key) => Box::new(DescentObjectField::new(key)),
        JsonPath::Current(Some(tail)) => Box::new(Current::new(process_path(tail, root))),
        JsonPath::Current(None) => Box::new(Current::none()),
        JsonPath::Index(index) => process_index(index, root),
        _ => Box::new(IdentityPath {})
    }
}

fn process_index<'a>(json_path_index: &'a JsonPathIndex, root: &'a Value) -> PathInstance<'a> {
    match json_path_index {
        JsonPathIndex::Single(index) => Box::new(ArrayIndex::new(*index)),
        JsonPathIndex::Slice(s, e, step) => Box::new(ArraySlice::new(*s, *e, *step)),
        JsonPathIndex::Union(elems) => Box::new(UnionIndex::from(elems, root)),
        JsonPathIndex::Filter(l, op, r) => Box::new(Filter::new(l, r, op, root)),
        _ => Box::new(IdentityPath {})
    }
}

fn process_operand<'a>(op: &'a Operand, root: &'a Value) -> PathInstance<'a> {
    match op {
        Operand::Static(v) => process_path(&JsonPath::Root, v),
        Operand::Dynamic(jp) => process_path(jp, root)
    }
}
