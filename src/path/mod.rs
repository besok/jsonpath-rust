use std::fmt::Debug;

use crate::{jsp_idx, jsp_obj, JsonPathValue};
use serde_json::{json, Value};

use crate::parser::model::{Function, JsonPath, JsonPathIndex, Operand};
pub use crate::path::index::{ArrayIndex, ArraySlice, Current, FilterPath, UnionIndex};
use crate::path::top::*;

/// The module is in charge of processing [[JsonPathIndex]] elements
mod index;
/// The module is a helper module providing the set of helping funcitons to process a json elements
mod json;
/// The module is responsible for processing of the [[JsonPath]] elements
mod top;

pub trait JsonLike {
    type Data;
    fn get(&self, key: &str) -> Option<&Self::Data>;
    fn itre(&self, pref: String) -> Vec<JsonPathValue<'_, Self::Data>>;
    fn array_len(&self) -> JsonPathValue<'static, Self::Data>;
    fn init_with_usize(cnt: usize) -> Self::Data;
    fn deep_flatten(&self, pref: String) -> Vec<(&Self::Data, String)>;
    fn deep_path_by_key<'a>(
        &'a self,
        key: ObjectField<'a, Self::Data>,
        pref: String,
    ) -> Vec<(&'a Self::Data, String)>;
    fn as_u64(&self) -> Option<u64>;
    fn as_array(&self) -> Option<&Vec<Self::Data>>;
    fn size<T>(left: Vec<&Self::Data>, right: Vec<&Self::Data>) -> bool;
}

impl JsonLike for Value {
    type Data = Value;

    fn get(&self, key: &str) -> Option<&Self::Data> {
        self.get(key)
    }

    fn itre(&self, pref: String) -> Vec<JsonPathValue<'_, Self::Data>> {
        let res = match self {
            Value::Array(elems) => {
                let mut res = vec![];
                for (idx, el) in elems.iter().enumerate() {
                    res.push(JsonPathValue::Slice(el, jsp_idx(&pref, idx)));
                }
                res
            }
            Value::Object(elems) => {
                let mut res = vec![];
                for (key, el) in elems.into_iter() {
                    res.push(JsonPathValue::Slice(el, jsp_obj(&pref, key)));
                }
                res
            }
            _ => vec![],
        };
        if res.is_empty() {
            vec![JsonPathValue::NoValue]
        } else {
            res
        }
    }
    fn array_len(&self) -> JsonPathValue<'static, Self::Data> {
        match self {
            Value::Array(elems) => JsonPathValue::NewValue(json!(elems.len())),
            _ => JsonPathValue::NoValue,
        }
    }

    fn init_with_usize(cnt: usize) -> Self::Data {
        json!(cnt)
    }
    fn deep_flatten(&self, pref: String) -> Vec<(&Self::Data, String)> {
        let mut acc = vec![];
        match self {
            Value::Object(elems) => {
                for (f, v) in elems.into_iter() {
                    let pref = jsp_obj(&pref, f);
                    acc.push((v, pref.clone()));
                    acc.append(&mut v.deep_flatten(pref));
                }
            }
            Value::Array(elems) => {
                for (i, v) in elems.iter().enumerate() {
                    let pref = jsp_idx(&pref, i);
                    acc.push((v, pref.clone()));
                    acc.append(&mut v.deep_flatten(pref));
                }
            }
            _ => (),
        }
        acc
    }
    fn deep_path_by_key<'a>(
        &'a self,
        key: ObjectField<'a, Self::Data>,
        pref: String,
    ) -> Vec<(&'a Self::Data, String)> {
        let mut result: Vec<(&'a Value, String)> =
            JsonPathValue::vec_as_pair(key.find(JsonPathValue::new_slice(self, pref.clone())));
        match self {
            Value::Object(elems) => {
                let mut next_levels: Vec<(&'a Value, String)> = elems
                    .into_iter()
                    .flat_map(|(k, v)| v.deep_path_by_key(key.clone(), jsp_obj(&pref, k)))
                    .collect();
                result.append(&mut next_levels);
                result
            }
            Value::Array(elems) => {
                let mut next_levels: Vec<(&'a Value, String)> = elems
                    .iter()
                    .enumerate()
                    .flat_map(|(i, v)| v.deep_path_by_key(key.clone(), jsp_idx(&pref, i)))
                    .collect();
                result.append(&mut next_levels);
                result
            }
            _ => result,
        }
    }

    fn as_u64(&self) -> Option<u64> {
        self.as_u64()
    }

    fn as_array(&self) -> Option<&Vec<Self::Data>> {
        self.as_array()
    }

    fn size<T>(left: Vec<&Self::Data>, right: Vec<&Self::Data>) -> bool {
        if let Some(Value::Number(n)) = right.first() {
            if let Some(sz) = n.as_f64() {
                for el in left.iter() {
                    match el {
                        Value::String(v) if v.len() == sz as usize => true,
                        Value::Array(elems) if elems.len() == sz as usize => true,
                        Value::Object(fields) if fields.len() == sz as usize => true,
                        _ => return false,
                    };
                }
                return true;
            }
        }
        false
    }
}

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
pub enum TopPaths<'a, T> {
    RootPointer(RootPointer<'a, T>),
    ObjectField(ObjectField<'a, T>),
    Chain(Chain<'a, T>),
    Wildcard(Wildcard<T>),
    DescentObject(DescentObject<'a, T>),
    DescentWildcard(DescentWildcard<T>),
    Current(Current<'a, T>),
    ArrayIndex(ArrayIndex),
    ArraySlice(ArraySlice),
    UnionIndex(UnionIndex<'a, T>),
    FilterPath(FilterPath<'a, T>),
    IdentityPath(IdentityPath),
    FnPath(FnPath<T>),
}

impl<'a, T> Path<'a> for TopPaths<'a, T>
where
    T: JsonLike<Data = T> + Default + Clone + Debug,
{
    type Data = T;

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
pub(crate) type PathInstance<'a, T> = Box<dyn Path<'a, Data = T> + 'a>;

/// The major method to process the top part of json part
pub(crate) fn json_path_instance<'a, T: JsonLike<Data = T>>(
    json_path: &'a JsonPath<T>,
    root: &'a T,
) -> TopPaths<'a, T>
where
    T: JsonLike<Data = T> + Default + Clone + Debug,
{
    match json_path {
        JsonPath::Root => TopPaths::RootPointer(RootPointer::new(root)),
        JsonPath::Field(key) => TopPaths::ObjectField(ObjectField::new(key)),
        JsonPath::Chain(chain) => TopPaths::Chain(Chain::from(chain, root)),
        JsonPath::Wildcard => TopPaths::Wildcard(Wildcard::new()),
        JsonPath::Descent(key) => TopPaths::DescentObject(DescentObject::new(key)),
        JsonPath::DescentW => TopPaths::DescentWildcard(DescentWildcard::new()),
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
        JsonPath::Fn(Function::Length) => TopPaths::FnPath(FnPath::new_size()),
    }
}

/// The method processes the operand inside the filter expressions
fn process_operand<'a, T>(op: &'a Operand<T>, root: &'a T) -> PathInstance<'a, T>
where
    T: JsonLike<Data = T> + Default + Clone + Debug,
{
    Box::new(match op {
        Operand::Static(v) => json_path_instance(&JsonPath::Root, v),
        Operand::Dynamic(jp) => json_path_instance(jp, root),
    })
}
