use std::fmt::Debug;

use crate::{jsp_idx, jsp_obj, JsonPathValue};
use regex::Regex;
use serde_json::{json, Value};

use crate::parser::model::{Function, JsonPath, JsonPathIndex, Operand};
pub use crate::path::index::{ArrayIndex, ArraySlice, Current, FilterPath, UnionIndex};
use crate::path::top::*;

/// The module is in charge of processing [[JsonPathIndex]] elements
mod index;
/// The module is responsible for processing of the [[JsonPath]] elements
mod top;

pub trait JsonLike:
    Default
    + Clone
    + Debug
    + for<'a> From<&'a str>
    + From<Vec<String>>
    + From<bool>
    + From<i64>
    + From<f64>
    + From<Vec<Self>>
    + From<String>
    + PartialEq
    + 'static
{
    /// Retrieves a reference to the value associated with the given key.
    fn get(&self, key: &str) -> Option<&Self>;

    /// Iterates over the elements with a given prefix and returns a vector of `JsonPathValue`.
    fn itre(&self, pref: String) -> Vec<JsonPathValue<'_, Self>>;

    /// Returns the length of the array as a `JsonPathValue`.
    fn array_len(&self) -> JsonPathValue<'static, Self>;

    /// Initializes an instance with a specific size.
    fn init_with_usize(cnt: usize) -> Self;

    /// Flattens nested structures and returns a vector of tuples containing references to the elements and their paths.
    fn deep_flatten(&self, pref: String) -> Vec<(&Self, String)>;

    /// Performs a deep search by key and returns a vector of tuples containing references to the elements and their paths.
    fn deep_path_by_key<'a>(
        &'a self,
        key: ObjectField<'a, Self>,
        pref: String,
    ) -> Vec<(&'a Self, String)>;

    /// Converts the element to an `Option<u64>`.
    fn as_u64(&self) -> Option<u64>;

    /// Checks if the element is an array.
    fn is_array(&self) -> bool;

    /// Converts the element to an `Option<&Vec<Self>>`.
    fn as_array(&self) -> Option<&Vec<Self>>;

    /// Compares the size of two vectors of references to elements.
    fn size(left: Vec<&Self>, right: Vec<&Self>) -> bool;

    /// Checks if the left vector is a subset of the right vector.
    fn sub_set_of(left: Vec<&Self>, right: Vec<&Self>) -> bool;

    /// Checks if any element in the left vector is present in the right vector.
    fn any_of(left: Vec<&Self>, right: Vec<&Self>) -> bool;

    /// Checks if the elements in the left vector match the regex pattern in the right vector.
    fn regex(left: Vec<&Self>, right: Vec<&Self>) -> bool;

    /// Checks if any element in the left vector is inside the right vector.
    fn inside(left: Vec<&Self>, right: Vec<&Self>) -> bool;

    /// Ensures the number on the left side is less than the number on the right side.
    fn less(left: Vec<&Self>, right: Vec<&Self>) -> bool;

    /// Compares elements for equality.
    fn eq(left: Vec<&Self>, right: Vec<&Self>) -> bool;

    /// Returns a null value.
    fn null() -> Self;

    /// Creates an array from a vector of elements.
    fn array(data: Vec<Self>) -> Self;
}

impl JsonLike for Value {
    fn is_array(&self) -> bool {
        self.is_array()
    }
    fn array(data: Vec<Self>) -> Self {
        Value::Array(data)
    }

    fn null() -> Self {
        Value::Null
    }

    fn get(&self, key: &str) -> Option<&Self> {
        self.get(key)
    }

    fn itre(&self, pref: String) -> Vec<JsonPathValue<'_, Self>> {
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
    fn array_len(&self) -> JsonPathValue<'static, Self> {
        match self {
            Value::Array(elems) => JsonPathValue::NewValue(json!(elems.len())),
            _ => JsonPathValue::NoValue,
        }
    }

    fn init_with_usize(cnt: usize) -> Self {
        json!(cnt)
    }
    fn deep_flatten(&self, pref: String) -> Vec<(&Self, String)> {
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
        key: ObjectField<'a, Self>,
        pref: String,
    ) -> Vec<(&'a Self, String)> {
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

    fn as_array(&self) -> Option<&Vec<Self>> {
        self.as_array()
    }

    fn size(left: Vec<&Self>, right: Vec<&Self>) -> bool {
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

    fn sub_set_of(left: Vec<&Self>, right: Vec<&Self>) -> bool {
        if left.is_empty() {
            return true;
        }
        if right.is_empty() {
            return false;
        }

        if let Some(elems) = left.first().and_then(|e| e.as_array()) {
            if let Some(Value::Array(right_elems)) = right.first() {
                if right_elems.is_empty() {
                    return false;
                }

                for el in elems {
                    let mut res = false;

                    for r in right_elems.iter() {
                        if el.eq(r) {
                            res = true
                        }
                    }
                    if !res {
                        return false;
                    }
                }
                return true;
            }
        }
        false
    }

    fn any_of(left: Vec<&Self>, right: Vec<&Self>) -> bool {
        if left.is_empty() {
            return true;
        }
        if right.is_empty() {
            return false;
        }

        if let Some(Value::Array(elems)) = right.first() {
            if elems.is_empty() {
                return false;
            }

            for el in left.iter() {
                if let Some(left_elems) = el.as_array() {
                    for l in left_elems.iter() {
                        for r in elems.iter() {
                            if l.eq(r) {
                                return true;
                            }
                        }
                    }
                } else {
                    for r in elems.iter() {
                        if el.eq(&r) {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    fn regex(left: Vec<&Self>, right: Vec<&Self>) -> bool {
        if left.is_empty() || right.is_empty() {
            return false;
        }

        match right.first() {
            Some(Value::String(str)) => {
                if let Ok(regex) = Regex::new(str) {
                    for el in left.iter() {
                        if let Some(v) = el.as_str() {
                            if regex.is_match(v) {
                                return true;
                            }
                        }
                    }
                }
                false
            }
            _ => false,
        }
    }

    fn inside(left: Vec<&Self>, right: Vec<&Self>) -> bool {
        if left.is_empty() {
            return false;
        }

        match right.first() {
            Some(Value::Array(elems)) => {
                for el in left.iter() {
                    if elems.contains(el) {
                        return true;
                    }
                }
                false
            }
            Some(Value::Object(elems)) => {
                for el in left.iter() {
                    for r in elems.values() {
                        if el.eq(&r) {
                            return true;
                        }
                    }
                }
                false
            }
            _ => false,
        }
    }

    /// ensure the number on the left side is less the number on the right side
    fn less(left: Vec<&Self>, right: Vec<&Self>) -> bool {
        if left.len() == 1 && right.len() == 1 {
            match (left.first(), right.first()) {
                (Some(Value::Number(l)), Some(Value::Number(r))) => l
                    .as_f64()
                    .and_then(|v1| r.as_f64().map(|v2| v1 < v2))
                    .unwrap_or(false),
                _ => false,
            }
        } else {
            false
        }
    }

    /// compare elements
    fn eq(left: Vec<&Self>, right: Vec<&Self>) -> bool {
        if left.len() != right.len() {
            false
        } else {
            left.iter().zip(right).map(|(a, b)| a.eq(&b)).all(|a| a)
        }
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
    ArrayIndex(ArrayIndex<T>),
    ArraySlice(ArraySlice<T>),
    UnionIndex(UnionIndex<'a, T>),
    FilterPath(FilterPath<'a, T>),
    IdentityPath(IdentityPath<T>),
    FnPath(FnPath<T>),
}

impl<'a, T> Path<'a> for TopPaths<'a, T>
where
    T: JsonLike,
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
pub(crate) fn json_path_instance<'a, T>(json_path: &'a JsonPath<T>, root: &'a T) -> TopPaths<'a, T>
where
    T: JsonLike,
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
        JsonPath::Empty => TopPaths::IdentityPath(IdentityPath::new()),
        JsonPath::Fn(Function::Length) => TopPaths::FnPath(FnPath::new_size()),
    }
}

/// The method processes the operand inside the filter expressions
fn process_operand<'a, T>(op: &'a Operand<T>, root: &'a T) -> PathInstance<'a, T>
where
    T: JsonLike,
{
    Box::new(match op {
        Operand::Static(v) => json_path_instance(&JsonPath::Root, v),
        Operand::Dynamic(jp) => json_path_instance(jp, root),
    })
}

#[cfg(test)]
mod tests {

    use crate::path::JsonLike;
    use serde_json::{json, Value};

    #[test]
    fn value_eq_test() {
        let left = json!({"value":42});
        let right = json!({"value":42});
        let right_uneq = json!([42]);
        assert!(&left.eq(&right));
        assert!(!&left.eq(&right_uneq));
    }

    #[test]
    fn vec_value_test() {
        let left = json!({"value":42});
        let left1 = json!(42);
        let left2 = json!([1, 2, 3]);
        let left3 = json!({"value2":[42],"value":[42]});

        let right = json!({"value":42});
        let right1 = json!(42);
        let right2 = json!([1, 2, 3]);
        let right3 = json!({"value":[42],"value2":[42]});

        assert!(JsonLike::eq(vec![&left], vec![&right]));

        assert!(!JsonLike::eq(vec![], vec![&right]));
        assert!(!JsonLike::eq(vec![&right], vec![]));

        assert!(JsonLike::eq(
            vec![&left, &left1, &left2, &left3],
            vec![&right, &right1, &right2, &right3]
        ));

        assert!(!JsonLike::eq(
            vec![&left1, &left, &left2, &left3],
            vec![&right, &right1, &right2, &right3]
        ));
    }

    #[test]
    fn less_value_test() {
        let left = json!(10);
        let right = json!(11);

        assert!(JsonLike::less(vec![&left], vec![&right]));
        assert!(!JsonLike::less(vec![&right], vec![&left]));

        let left = json!(-10);
        let right = json!(-11);

        assert!(!JsonLike::less(vec![&left], vec![&right]));
        assert!(JsonLike::less(vec![&right], vec![&left]));

        let left = json!(-10.0);
        let right = json!(-11.0);

        assert!(!JsonLike::less(vec![&left], vec![&right]));
        assert!(JsonLike::less(vec![&right], vec![&left]));

        assert!(!JsonLike::less(vec![], vec![&right]));
        assert!(!JsonLike::less(vec![&right, &right], vec![&left]));
    }

    #[test]
    fn regex_test() {
        let right = json!("[a-zA-Z]+[0-9]#[0-9]+");
        let left1 = json!("a11#");
        let left2 = json!("a1#1");
        let left3 = json!("a#11");
        let left4 = json!("#a11");

        assert!(JsonLike::regex(
            vec![&left1, &left2, &left3, &left4],
            vec![&right]
        ));
        assert!(!JsonLike::regex(vec![&left1, &left3, &left4], vec![&right]))
    }

    #[test]
    fn any_of_test() {
        let right = json!([1, 2, 3, 4, 5, 6]);
        let left = json!([1, 100, 101]);
        assert!(JsonLike::any_of(vec![&left], vec![&right]));

        let left = json!([11, 100, 101]);
        assert!(!JsonLike::any_of(vec![&left], vec![&right]));

        let left1 = json!(1);
        let left2 = json!(11);
        assert!(JsonLike::any_of(vec![&left1, &left2], vec![&right]));
    }

    #[test]
    fn sub_set_of_test() {
        let left1 = json!(1);
        let left2 = json!(2);
        let left3 = json!(3);
        let left40 = json!(40);
        let right = json!([1, 2, 3, 4, 5, 6]);
        assert!(JsonLike::sub_set_of(
            vec![&Value::Array(vec![
                left1.clone(),
                left2.clone(),
                left3.clone()
            ])],
            vec![&right]
        ));
        assert!(!JsonLike::sub_set_of(
            vec![&Value::Array(vec![left1, left2, left3, left40])],
            vec![&right]
        ));
    }

    #[test]
    fn size_test() {
        let left1 = json!("abc");
        let left2 = json!([1, 2, 3]);
        let left3 = json!([1, 2, 3, 4]);
        let right = json!(3);
        let right1 = json!(4);
        assert!(JsonLike::size(vec![&left1], vec![&right]));
        assert!(JsonLike::size(vec![&left2], vec![&right]));
        assert!(!JsonLike::size(vec![&left3], vec![&right]));
        assert!(JsonLike::size(vec![&left3], vec![&right1]));
    }
}
