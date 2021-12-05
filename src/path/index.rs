use crate::path::{Path, PathInstance, json_path_instance, process_operand};
use serde_json::Value;
use crate::parser::model::{JsonPath, FilterSign, Operand};
use crate::path::json::*;
use serde_json::value::Value::{Array, Object};
use crate::path::top::ObjectField;

/// process the slice like [start:end:step]
#[derive(Debug)]
pub(crate) struct ArraySlice {
    start_index: i32,
    end_index: i32,
    step: usize,
}

impl ArraySlice {
    pub(crate) fn new(start_index: i32,
                      end_index: i32,
                      step: usize, ) -> ArraySlice {
        ArraySlice { start_index, end_index, step }
    }

    fn end(&self, len: i32) -> Option<usize> {
        if self.end_index >= 0 {
            if self.end_index > len { None } else { Some(self.end_index as usize) }
        } else if self.end_index < -len { None } else {
            Some((len - (-self.end_index)) as usize)
        }
    }

    fn start(&self, len: i32) -> Option<usize> {
        if self.start_index >= 0 {
            if self.start_index > len { None } else { Some(self.start_index as usize) }
        } else if self.start_index < -len { None } else {
            Some((len - -self.start_index) as usize)
        }
    }

    fn process<'a, T>(&self, elements: &'a [T]) -> Vec<&'a T> {
        let len = elements.len() as i32;
        let mut filtered_elems: Vec<&T> = vec![];
        match (self.start(len), self.end(len)) {
            (Some(start_idx), Some(end_idx)) => {
                let end_idx = if end_idx == 0 { elements.len() } else { end_idx };
                for idx in (start_idx..end_idx).step_by(self.step) {
                    if let Some(v) = elements.get(idx) {
                        filtered_elems.push(v)
                    }
                }
                filtered_elems
            }
            _ => filtered_elems
        }
    }
}

impl<'a> Path<'a> for ArraySlice {
    type Data = Value;

    fn find(&self, data: &'a Self::Data) -> Vec<&'a Self::Data> {
        data.as_array()
            .map(|elems| self.process(elems))
            .unwrap_or_default()
    }
}

/// process the simple index like [index]
pub(crate) struct ArrayIndex {
    index: usize,
}

impl ArrayIndex {
    pub(crate) fn new(index: usize) -> Self {
        ArrayIndex { index }
    }
}

impl<'a> Path<'a> for ArrayIndex {
    type Data = Value;

    fn find(&self, data: &'a Self::Data) -> Vec<&'a Self::Data> {
        data.as_array()
            .and_then(|elems| elems.get(self.index))
            .map(|e| vec![e])
            .unwrap_or_default()
    }
}

/// process @ element
pub(crate) struct Current<'a> {
    tail: Option<PathInstance<'a>>,
}

impl<'a> Current<'a> {
    pub(crate) fn from(jp: &'a JsonPath, root: &'a Value) -> Self {
        match jp {
            JsonPath::Empty => Current::none(),
            tail => Current::new(json_path_instance(tail, root))
        }
    }
    pub(crate) fn new(tail: PathInstance<'a>) -> Self {
        Current { tail: Some(tail) }
    }
    pub(crate) fn none() -> Self {
        Current { tail: None }
    }
}

impl<'a> Path<'a> for Current<'a> {
    type Data = Value;

    fn find(&self, data: &'a Self::Data) -> Vec<&'a Value> {
        self.tail.as_ref().map(|p| p.find(data)).unwrap_or_else(|| vec![data])
    }
}

/// the list of indexes like [1,2,3]
pub(crate) struct UnionIndex<'a> {
    indexes: Vec<PathInstance<'a>>,
}

impl<'a> UnionIndex<'a> {
    pub fn from_indexes(elems: &'a [Value]) -> Self {
        let mut indexes: Vec<PathInstance<'a>> = vec![];

        for idx in elems.iter() {
            indexes.push(Box::new(ArrayIndex::new(idx.as_u64().unwrap() as usize)))
        }

        UnionIndex::new(indexes)
    }
    pub fn from_keys(elems: &'a [String]) -> Self {
        let mut indexes: Vec<PathInstance<'a>> = vec![];

        for key in elems.iter() {
            indexes.push(Box::new(ObjectField::new(key)))
        }

        UnionIndex::new(indexes)
    }

    pub fn new(indexes: Vec<PathInstance<'a>>) -> Self {
        UnionIndex { indexes }
    }
}

impl<'a> Path<'a> for UnionIndex<'a> {
    type Data = Value;

    fn find(&self, data: &'a Self::Data) -> Vec<&'a Self::Data> {
        self.indexes.iter().flat_map(|e| e.find(data)).collect()
    }
}

/// process filter element like [?(op sign op)]
pub(crate) struct Filter<'a> {
    left: PathInstance<'a>,
    right: PathInstance<'a>,
    op: &'a FilterSign,
}

impl<'a> Filter<'a> {
    pub(crate) fn new(left: &'a Operand, right: &'a Operand, op: &'a FilterSign, root: &'a Value) -> Self {
        Filter {
            left: process_operand(left, root),
            right: process_operand(right, root),
            op,
        }
    }

    fn or(one: &'a FilterSign, two: &'a FilterSign, left: Vec<&'a Value>, right: Vec<&'a Value>) -> bool {
        Filter::process(one, left.clone(), right.clone())
            || Filter::process(two, left.clone(), right.clone())
    }

    fn process(op: &'a FilterSign, left: Vec<&'a Value>, right: Vec<&'a Value>) -> bool {
        match op {
            FilterSign::Equal => eq(left, right),
            FilterSign::Unequal => !Filter::process(&FilterSign::Equal, left, right),
            FilterSign::Less => less(left, right),
            FilterSign::LeOrEq => Filter::or(&FilterSign::Less, &FilterSign::Equal, left, right),
            FilterSign::Greater => !Filter::process(&FilterSign::LeOrEq, left, right),
            FilterSign::GrOrEq => !Filter::process(&FilterSign::Less, left, right),
            FilterSign::Regex => regex(left, right),
            FilterSign::In => inside(left, right),
            FilterSign::Nin => !Filter::process(&FilterSign::In, left, right),
            FilterSign::NoneOf => !Filter::process(&FilterSign::AnyOf, left, right),
            FilterSign::AnyOf => any_of(left, right),
            FilterSign::SubSetOf => sub_set_of(left, right),
            FilterSign::Exists => !left.is_empty(),
            FilterSign::Size => size(left, right)
        }
    }
}

impl<'a> Path<'a> for Filter<'a> {
    type Data = Value;

    fn find(&self, data: &'a Self::Data) -> Vec<&'a Self::Data> {
        let mut res: Vec<&Value> = vec![];

        match data {
            Array(elems) => {
                for el in elems.iter() {
                    if Filter::process(self.op, self.left.find(el), self.right.find(el)) {
                        res.push(el)
                    }
                }
                res
            }
            Object(pairs) => {
                for el in pairs.values() {
                    if Filter::process(self.op, self.left.find(el), self.right.find(el)) {
                        res.push(el)
                    }
                }
                res
            }
            _ => vec![]
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;
    use serde_json::json;
    use crate::parser::model::{JsonPath, JsonPathIndex, Operand, FilterSign};
    use crate::path::index::{ArraySlice, ArrayIndex};
    use crate::path::{Path, json_path_instance};

    #[test]
    fn array_slice_end_start_test() {
        let array = vec![0, 1, 2, 3, 4, 5];
        let len = array.len() as i32;
        let mut slice = ArraySlice::new(0, 0, 0);

        assert_eq!(slice.start(len).unwrap(), 0);
        slice.start_index = 1;

        assert_eq!(slice.start(len).unwrap(), 1);

        slice.start_index = 2;
        assert_eq!(slice.start(len).unwrap(), 2);

        slice.start_index = 5;
        assert_eq!(slice.start(len).unwrap(), 5);

        slice.start_index = 7;
        assert_eq!(slice.start(len), None);

        slice.start_index = -1;
        assert_eq!(slice.start(len).unwrap(), 5);

        slice.start_index = -5;
        assert_eq!(slice.start(len).unwrap(), 1);

        slice.end_index = 0;
        assert_eq!(slice.end(len).unwrap(), 0);

        slice.end_index = 5;
        assert_eq!(slice.end(len).unwrap(), 5);

        slice.end_index = -1;
        assert_eq!(slice.end(len).unwrap(), 5);

        slice.end_index = -5;
        assert_eq!(slice.end(len).unwrap(), 1);
    }

    #[test]
    fn slice_test() {
        let array = json!([0,1,2,3,4,5,6,7,8,9,10]);

        let mut slice = ArraySlice::new(0, 6, 2);
        assert_eq!(slice.find(&array), vec![&json!(0), &json!(2), &json!(4)]);

        slice.step = 3;
        assert_eq!(slice.find(&array), vec![&json!(0), &json!(3)]);

        slice.start_index = -1;
        slice.end_index = 1;

        assert!(slice.find(&array).is_empty());

        slice.start_index = -10;
        slice.end_index = 10;

        assert_eq!(slice.find(&array), vec![&json!(1), &json!(4), &json!(7)]);
    }

    #[test]
    fn index_test() {
        let array = json!([0,1,2,3,4,5,6,7,8,9,10]);

        let mut index = ArrayIndex::new(0);

        assert_eq!(index.find(&array), vec![&json!(0)]);
        index.index = 10;
        assert_eq!(index.find(&array), vec![&json!(10)]);
        index.index = 100;
        assert!(index.find(&array).is_empty());
    }

    #[test]
    fn current_test() {
        let json = json!(
        {
            "object":{
                "field_1":[1,2,3],
                "field_2":42,
                "field_3":{"a":"b"}

            }
        });

        let root = JsonPath::Root;
        let object = JsonPath::Field(String::from("object"));
        let cur = JsonPath::Current(Box::new(JsonPath::Empty));

        let chain = vec![root.clone(), object.clone(), cur.clone()];
        let chain = JsonPath::Chain(chain);

        let path_inst = json_path_instance(&chain, &json);
        let res = json!({
                "field_1":[1,2,3],
                "field_2":42,
                "field_3":{"a":"b"}
            });

        let expected_res = vec![&res];
        assert_eq!(path_inst.find(&json), expected_res);

        let field_3 = JsonPath::Field(String::from("field_3"));
        let field_a = JsonPath::Field(String::from("a"));
        let chain_in = vec![field_3, field_a];
        let chain_in = JsonPath::Chain(chain_in);
        let cur = JsonPath::Current(Box::new(chain_in));

        let chain = vec![root.clone(), object.clone(), cur.clone()];
        let chain = JsonPath::Chain(chain);

        let path_inst = json_path_instance(&chain, &json);
        let res1 = json!("b");

        let expected_res = vec![&res1];
        assert_eq!(path_inst.find(&json), expected_res);
    }

    #[test]
    fn filter_exist_test() {
        let json = json!({
            "key":[{"field":[1,2,3,4,5]},{"field":42}],
        });


        let field = JsonPath::Field(String::from("field"));
        let cur = JsonPath::Current(Box::new(field.clone()));
        let operand = Operand::Dynamic(Box::new(cur));
        let empty_operand = Operand::Static(Value::Null);


        let root = JsonPath::Root;
        let key = JsonPath::Field(String::from("key"));
        let filter = JsonPathIndex::Filter(operand, FilterSign::Exists, empty_operand);
        let index = JsonPath::Index(filter);

        let chain = vec![root.clone(), key.clone(), index.clone(), field.clone()];
        let chain = JsonPath::Chain(chain);

        let path_inst = json_path_instance(&chain, &json);

        let exp1 = json!([1,2,3,4,5]);
        let exp2 = json!(42);
        let expected_res = vec![&exp1, &exp2];
        assert_eq!(path_inst.find(&json), expected_res)
    }

    #[test]
    fn filter_gr_test() {
        let json = json!({
                "key":[
                    {"field":1},
                    {"field":10},
                    {"field":5},
                    {"field":1},
                ]
            });


        let field = JsonPath::Field(String::from("field"));
        let cur = JsonPath::Current(Box::new(field));
        let operand = Operand::Dynamic(Box::new(cur));
        let right_operand = Operand::Static(json!(1));


        let root = JsonPath::Root;
        let key = JsonPath::Field(String::from("key"));
        let filter = JsonPathIndex::Filter(operand, FilterSign::Greater, right_operand);
        let index = JsonPath::Index(filter);

        let chain = vec![root, key, index];
        let chain = JsonPath::Chain(chain);

        let path_inst = json_path_instance(&chain, &json);

        let exp1 = json!( {"field":10});
        let exp2 = json!( {"field":5});
        let expected_res = vec![&exp1, &exp2];
        assert_eq!(path_inst.find(&json), expected_res)
    }

    #[test]
    fn filter_regex_test() {
        let json = json!({
                "key":[
                    {"field":"a11#"},
                    {"field":"a1#1"},
                    {"field":"a#11"},
                    {"field":"#a11"},
                ]
            });


        let field = JsonPath::Field(String::from("field"));
        let cur = JsonPath::Current(Box::new(field));
        let operand = Operand::Dynamic(Box::new(cur));
        let right_operand = Operand::Static(json!("[a-zA-Z]+[0-9]#[0-9]+"));


        let root = JsonPath::Root;
        let key = JsonPath::Field(String::from("key"));
        let filter = JsonPathIndex::Filter(operand, FilterSign::Regex, right_operand);
        let index = JsonPath::Index(filter);

        let chain = vec![root, key, index];
        let chain = JsonPath::Chain(chain);

        let path_inst = json_path_instance(&chain, &json);

        let exp2 = json!( {"field":"a1#1"});
        let expected_res = vec![&exp2];
        assert_eq!(path_inst.find(&json), expected_res)
    }

    #[test]
    fn filter_any_of_test() {
        let json = json!({
                "key":[
                    {"field":"a11#"},
                    {"field":"a1#1"},
                    {"field":"a#11"},
                    {"field":"#a11"},
                ]
            });


        let field = JsonPath::Field(String::from("field"));
        let cur = JsonPath::Current(Box::new(field));
        let operand = Operand::Dynamic(Box::new(cur));
        let right_operand = Operand::Static(json!(["a11#","aaa","111"]));


        let root = JsonPath::Root;
        let key = JsonPath::Field(String::from("key"));
        let filter = JsonPathIndex::Filter(operand, FilterSign::AnyOf, right_operand);
        let index = JsonPath::Index(filter);

        let chain = vec![root, key, index];
        let chain = JsonPath::Chain(chain);

        let path_inst = json_path_instance(&chain, &json);

        let exp2 = json!( {"field":"a11#"});
        let expected_res = vec![&exp2];
        assert_eq!(path_inst.find(&json), expected_res)
    }

    #[test]
    fn size_test() {
        let json = json!({
                "key":[
                    {"field":"aaaa"},
                    {"field":"bbb"},
                    {"field":"cc"},
                    {"field":"dddd"},
                    {"field":[1,1,1,1]},
                ]
            });


        let field = JsonPath::Field(String::from("field"));
        let cur = JsonPath::Current(Box::new(field));
        let operand = Operand::Dynamic(Box::new(cur));
        let right_operand = Operand::Static(json!(4));


        let root = JsonPath::Root;
        let key = JsonPath::Field(String::from("key"));
        let filter = JsonPathIndex::Filter(operand, FilterSign::Size, right_operand);
        let index = JsonPath::Index(filter);

        let chain = vec![root, key, index];
        let chain = JsonPath::Chain(chain);

        let path_inst = json_path_instance(&chain, &json);

        let exp2 = json!( {"field":"aaaa"});
        let exp3 = json!( {"field":"dddd"});
        let exp4 = json!( {"field":[1,1,1,1]});
        let expected_res = vec![&exp2, &exp3, &exp4];
        assert_eq!(path_inst.find(&json), expected_res)
    }
}