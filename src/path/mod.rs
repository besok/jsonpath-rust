
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


/// just to create a slice of data whether it is slice or ref
#[macro_export]
macro_rules! path_value {
    (&$v:expr) =>{
        PathValue::Ref(&$v)
    };
    ($(&$v:expr),+ $(,)?) =>{
        {
        let mut res = Vec::new();
        $(
           res.push(path_value!(&$v));
        )+
        res
        }
    };
    ($v:expr) =>{
        PathValue::Slice($v)
    };

}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum PathValue<'a, Data> {
    Ref(&'a Data),
    Slice(Data),
}

impl<'a, Data> From<&'a Data> for PathValue<'a, Data> {
    fn from(data: &'a Data) -> Self {
        PathValue::Ref(data)
    }
}

impl<'a, D> PathValue<'a, D> {
    fn map_ref<F>(self, mapper: F) -> Vec<PathValue<'a, D>>
        where F: FnOnce(&'a D) -> Vec<&'a D>
    {
        match self {
            PathValue::Ref(r) => {
                mapper(r)
                    .into_iter()
                    .map(PathValue::Ref)
                    .collect()
            }
            PathValue::Slice(_) => vec![]
        }
    }
    pub fn ref_into_vec(input: Vec<PathValue<'a, D>>) -> Vec<&'a D> {
        input
            .into_iter()
            .filter_map(|v| match v {
                PathValue::Ref(el) => Some(el),
                _ => None
            })
            .collect()
    }
}

/// The trait defining the behaviour of processing every separated element.
/// type Data usually stands for json [[Value]]
/// The trait also requires to have a root json to process.
/// It needs in case if in the filter there will be a pointer to the absolute path
pub trait Path<'a> {
    type Data;
    fn find(&self, input: PathValue<'a, Self::Data>) -> Vec<PathValue<'a, Self::Data>>;
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
