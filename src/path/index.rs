use crate::parser::model::{FilterExpression, FilterSign, JsonPath};
use crate::path::json::*;
use crate::path::top::ObjectField;
use crate::path::{json_path_instance, process_operand, JsonPathValue, Path, PathInstance};
use crate::JsonPathValue::{NoValue, Slice};
use serde_json::value::Value::Array;
use serde_json::Value;

/// process the slice like [start:end:step]
#[derive(Debug)]
pub(crate) struct ArraySlice {
    start_index: i32,
    end_index: i32,
    step: usize,
}

impl ArraySlice {
    pub(crate) fn new(start_index: i32, end_index: i32, step: usize) -> ArraySlice {
        ArraySlice {
            start_index,
            end_index,
            step,
        }
    }

    fn end(&self, len: i32) -> Option<usize> {
        if self.end_index >= 0 {
            if self.end_index > len {
                None
            } else {
                Some(self.end_index as usize)
            }
        } else if self.end_index < -len {
            None
        } else {
            Some((len - (-self.end_index)) as usize)
        }
    }

    fn start(&self, len: i32) -> Option<usize> {
        if self.start_index >= 0 {
            if self.start_index > len {
                None
            } else {
                Some(self.start_index as usize)
            }
        } else if self.start_index < -len {
            None
        } else {
            Some((len - -self.start_index) as usize)
        }
    }

    fn process<'a, T>(&self, elements: &'a [T]) -> Vec<&'a T> {
        let len = elements.len() as i32;
        let mut filtered_elems: Vec<&T> = vec![];
        match (self.start(len), self.end(len)) {
            (Some(start_idx), Some(end_idx)) => {
                let end_idx = if end_idx == 0 {
                    elements.len()
                } else {
                    end_idx
                };
                for idx in (start_idx..end_idx).step_by(self.step) {
                    if let Some(v) = elements.get(idx) {
                        filtered_elems.push(v)
                    }
                }
                filtered_elems
            }
            _ => filtered_elems,
        }
    }
}

impl<'a> Path<'a> for ArraySlice {
    type Data = Value;

    fn find(&self, input: JsonPathValue<'a, Self::Data>) -> Vec<JsonPathValue<'a, Self::Data>> {
        input.flat_map_slice(|data| {
            data.as_array()
                .map(|elems| self.process(elems))
                .and_then(|v| {
                    if v.is_empty() {
                        None
                    } else {
                        Some(JsonPathValue::map_vec(v))
                    }
                })
                .unwrap_or_else(|| vec![NoValue])
        })
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

    fn find(&self, input: JsonPathValue<'a, Self::Data>) -> Vec<JsonPathValue<'a, Self::Data>> {
        input.flat_map_slice(|data| {
            data.as_array()
                .and_then(|elems| elems.get(self.index))
                .map(|e| vec![e.into()])
                .unwrap_or_else(|| vec![NoValue])
        })
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
            tail => Current::new(json_path_instance(tail, root)),
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

    fn find(&self, input: JsonPathValue<'a, Self::Data>) -> Vec<JsonPathValue<'a, Self::Data>> {
        self.tail
            .as_ref()
            .map(|p| p.find(input.clone()))
            .unwrap_or_else(|| vec![input])
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

    fn find(&self, input: JsonPathValue<'a, Self::Data>) -> Vec<JsonPathValue<'a, Self::Data>> {
        self.indexes
            .iter()
            .flat_map(|e| e.find(input.clone()))
            .collect()
    }
}

/// process filter element like [?(op sign op)]
pub enum FilterPath<'a> {
    Filter {
        left: PathInstance<'a>,
        right: PathInstance<'a>,
        op: &'a FilterSign,
    },
    Or {
        left: PathInstance<'a>,
        right: PathInstance<'a>,
    },
    And {
        left: PathInstance<'a>,
        right: PathInstance<'a>,
    },
}

impl<'a> FilterPath<'a> {
    pub(crate) fn new(expr: &'a FilterExpression, root: &'a Value) -> Self {
        match expr {
            FilterExpression::Atom(left, op, right) => FilterPath::Filter {
                left: process_operand(left, root),
                right: process_operand(right, root),
                op,
            },
            FilterExpression::And(l, r) => FilterPath::And {
                left: Box::new(FilterPath::new(l, root)),
                right: Box::new(FilterPath::new(r, root)),
            },
            FilterExpression::Or(l, r) => FilterPath::Or {
                left: Box::new(FilterPath::new(l, root)),
                right: Box::new(FilterPath::new(r, root)),
            },
        }
    }
    fn compound(
        one: &'a FilterSign,
        two: &'a FilterSign,
        left: Vec<JsonPathValue<Value>>,
        right: Vec<JsonPathValue<Value>>,
    ) -> bool {
        FilterPath::process_atom(one, left.clone(), right.clone())
            || FilterPath::process_atom(two, left, right)
    }
    fn process_atom(
        op: &'a FilterSign,
        left: Vec<JsonPathValue<Value>>,
        right: Vec<JsonPathValue<Value>>,
    ) -> bool {
        match op {
            FilterSign::Equal => eq(
                JsonPathValue::into_data(left),
                JsonPathValue::into_data(right),
            ),
            FilterSign::Unequal => !FilterPath::process_atom(&FilterSign::Equal, left, right),
            FilterSign::Less => less(
                JsonPathValue::into_data(left),
                JsonPathValue::into_data(right),
            ),
            FilterSign::LeOrEq => {
                FilterPath::compound(&FilterSign::Less, &FilterSign::Equal, left, right)
            }
            FilterSign::Greater => !FilterPath::process_atom(&FilterSign::LeOrEq, left, right),
            FilterSign::GrOrEq => !FilterPath::process_atom(&FilterSign::Less, left, right),
            FilterSign::Regex => regex(
                JsonPathValue::into_data(left),
                JsonPathValue::into_data(right),
            ),
            FilterSign::In => inside(
                JsonPathValue::into_data(left),
                JsonPathValue::into_data(right),
            ),
            FilterSign::Nin => !FilterPath::process_atom(&FilterSign::In, left, right),
            FilterSign::NoneOf => !FilterPath::process_atom(&FilterSign::AnyOf, left, right),
            FilterSign::AnyOf => any_of(
                JsonPathValue::into_data(left),
                JsonPathValue::into_data(right),
            ),
            FilterSign::SubSetOf => sub_set_of(
                JsonPathValue::into_data(left),
                JsonPathValue::into_data(right),
            ),
            FilterSign::Exists => !JsonPathValue::into_data(left).is_empty(),
            FilterSign::Size => size(
                JsonPathValue::into_data(left),
                JsonPathValue::into_data(right),
            ),
        }
    }

    fn process(&self, curr_el: &'a Value) -> bool {
        match self {
            FilterPath::Filter { left, right, op } => {
                FilterPath::process_atom(op, left.find(Slice(curr_el)), right.find(Slice(curr_el)))
            }
            FilterPath::Or { left, right } => {
                if !JsonPathValue::into_data(left.find(Slice(curr_el))).is_empty() {
                    true
                } else {
                    !JsonPathValue::into_data(right.find(Slice(curr_el))).is_empty()
                }
            }
            FilterPath::And { left, right } => {
                if JsonPathValue::into_data(left.find(Slice(curr_el))).is_empty() {
                    false
                } else {
                    !JsonPathValue::into_data(right.find(Slice(curr_el))).is_empty()
                }
            }
        }
    }
}

impl<'a> Path<'a> for FilterPath<'a> {
    type Data = Value;

    fn find(&self, input: JsonPathValue<'a, Self::Data>) -> Vec<JsonPathValue<'a, Self::Data>> {
        input.flat_map_slice(|data| {
            let mut res = vec![];
            match data {
                Array(elems) => {
                    for el in elems.iter() {
                        if self.process(el) {
                            res.push(Slice(el))
                        }
                    }
                }
                el => {
                    if self.process(el) {
                        res.push(Slice(el))
                    }
                }
            }
            if res.is_empty() {
                vec![NoValue]
            } else {
                res
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::model::{FilterExpression, FilterSign, JsonPath, JsonPathIndex, Operand};
    use crate::path::index::{ArrayIndex, ArraySlice};
    use crate::path::JsonPathValue;
    use crate::path::{json_path_instance, Path};
    use crate::JsonPathValue::NoValue;
    use crate::{chain, filter, idx, json_path_value, op, path};
    use serde_json::json;

    #[test]
    fn array_slice_end_start_test() {
        let array = [0, 1, 2, 3, 4, 5];
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
        let array = json!([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

        let mut slice = ArraySlice::new(0, 6, 2);
        let j1 = json!(0);
        let j2 = json!(2);
        let j4 = json!(4);
        assert_eq!(slice.find((&array).into()), json_path_value![&j1, &j2, &j4]);

        slice.step = 3;
        let j0 = json!(0);
        let j3 = json!(3);
        assert_eq!(
            slice.find(json_path_value!(&array)),
            json_path_value![&j0, &j3]
        );

        slice.start_index = -1;
        slice.end_index = 1;

        assert_eq!(slice.find((&array).into()), vec![NoValue]);

        slice.start_index = -10;
        slice.end_index = 10;

        let j1 = json!(1);
        let j4 = json!(4);
        let j7 = json!(7);

        assert_eq!(slice.find((&array).into()), json_path_value![&j1, &j4, &j7]);
    }

    #[test]
    fn index_test() {
        let array = json!([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

        let mut index = ArrayIndex::new(0);
        let j0 = json!(0);
        let j10 = json!(10);
        assert_eq!(index.find((&array).into()), json_path_value![&j0,]);
        index.index = 10;
        assert_eq!(index.find((&array).into()), json_path_value![&j10,]);
        index.index = 100;
        assert_eq!(index.find((&array).into()), vec![NoValue]);
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

        let chain = chain!(path!($), path!("object"), path!(@));

        let path_inst = json_path_instance(&chain, &json);
        let res = json!({
            "field_1":[1,2,3],
            "field_2":42,
            "field_3":{"a":"b"}
        });

        let expected_res = vec![json_path_value!(&res)];
        assert_eq!(path_inst.find(json_path_value!(&json)), expected_res);

        let cur = path!(@,path!("field_3"),path!("a"));
        let chain = chain!(path!($), path!("object"), cur);

        let path_inst = json_path_instance(&chain, &json);
        let res1 = json!("b");

        let expected_res = vec![(&res1).into()];
        assert_eq!(path_inst.find(json_path_value!(&json)), expected_res);
    }

    #[test]
    fn filter_exist_test() {
        let json = json!({
            "threshold" : 3,
            "key":[{"field":[1,2,3,4,5],"field1":[7]},{"field":42}],
        });

        let index = path!(idx!(?filter!(op!(path!(@, path!("field"))), "exists", op!())));
        let chain = chain!(path!($), path!("key"), index, path!("field"));

        let path_inst = json_path_instance(&chain, &json);

        let exp1 = json!([1, 2, 3, 4, 5]);
        let exp2 = json!(42);
        let expected_res = vec![json_path_value!(&exp1), json_path_value!(&exp2)];
        assert_eq!(path_inst.find(json_path_value!(&json)), expected_res)
    }

    #[test]
    fn filter_gr_test() {
        let json = json!({
            "threshold" : 4,
            "key":[
                {"field":1},
                {"field":10},
                {"field":5},
                {"field":1},
            ]
        });

        let index = path!(
            idx!(?filter!(op!(path!(@, path!("field"))), ">", op!(chain!(path!($), path!("threshold")))))
        );

        let chain = chain!(path!($), path!("key"), index);

        let path_inst = json_path_instance(&chain, &json);

        let exp1 = json!( {"field":10});
        let exp2 = json!( {"field":5});
        let expected_res = json_path_value![&exp1, &exp2];
        assert_eq!(path_inst.find((&json).into()), expected_res)
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

        let index = idx!(?filter!(op!(path!(@,path!("field"))),"~=", op!("[a-zA-Z]+[0-9]#[0-9]+")));
        let chain = chain!(path!($), path!("key"), path!(index));

        let path_inst = json_path_instance(&chain, &json);

        let exp2 = json!( {"field":"a1#1"});
        let expected_res = json_path_value![&exp2,];
        assert_eq!(path_inst.find((&json).into()), expected_res)
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

        let index = idx!(?filter!(
            op!(path!(@,path!("field"))),
            "anyOf",
            op!(s ["a11#","aaa","111"])
        ));

        let chain = chain!(path!($), JsonPath::Field(String::from("key")), path!(index));

        let path_inst = json_path_instance(&chain, &json);

        let exp2 = json!( {"field":"a11#"});
        let expected_res = json_path_value![&exp2,];
        assert_eq!(path_inst.find((&json).into()), expected_res)
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

        let index = idx!(?filter!(op!(path!(@, path!("field"))),"size",op!(4)));
        let chain = chain!(path!($), path!("key"), path!(index));
        let path_inst = json_path_instance(&chain, &json);

        let f1 = json!( {"field":"aaaa"});
        let f2 = json!( {"field":"dddd"});
        let f3 = json!( {"field":[1,1,1,1]});

        let expected_res = json_path_value![&f1, &f2, &f3];
        assert_eq!(path_inst.find((&json).into()), expected_res)
    }

    #[test]
    fn nested_filter_test() {
        let json = json!({
            "obj":{
                "id":1,
                "not_id": 2,
                "more_then_id" :3
            }
        });
        let index = idx!(?filter!(
            op!(path!(@,path!("not_id"))), "==",op!(2)
        ));
        let chain = chain!(path!($), path!("obj"), path!(index));
        let path_inst = json_path_instance(&chain, &json);
        let js = json!({
            "id":1,
            "not_id": 2,
            "more_then_id" :3
        });
        assert_eq!(path_inst.find((&json).into()), json_path_value![&js,])
    }

    #[test]
    fn or_arr_test() {
        let json = json!({
            "key":[
                {"city":"London","capital":true, "size": "big"},
                {"city":"Berlin","capital":true,"size": "big"},
                {"city":"Tokyo","capital":true,"size": "big"},
                {"city":"Moscow","capital":true,"size": "big"},
                {"city":"Athlon","capital":false,"size": "small"},
                {"city":"Dortmund","capital":false,"size": "big"},
                {"city":"Dublin","capital":true,"size": "small"},
            ]
        });
        let index = idx!(?filter!(
            filter!(op!(path!(@,path!("capital"))), "==", op!(false)),
            ||,
            filter!(op!(path!(@,path!("size"))), "==", op!("small"))
        )
        );
        let chain = chain!(path!($), path!("key"), path!(index), path!("city"));
        let path_inst = json_path_instance(&chain, &json);
        let a = json!("Athlon");
        let d = json!("Dortmund");
        let dd = json!("Dublin");
        assert_eq!(
            path_inst.find((&json).into()),
            json_path_value![&a, &d, &dd]
        )
    }

    #[test]
    fn or_obj_test() {
        let json = json!({
            "key":{
             "id":1,
             "name":"a",
             "another":"b"
            }
        });
        let index = idx!(?filter!(
            filter!(op!(path!(@,path!("name"))), "==", op!("a")),
            ||,
            filter!(op!(path!(@,path!("another"))), "==", op!("b"))
        )
        );
        let chain = chain!(path!($), path!("key"), path!(index), path!("id"));
        let path_inst = json_path_instance(&chain, &json);
        let j1 = json!(1);
        assert_eq!(path_inst.find((&json).into()), json_path_value![&j1,])
    }

    #[test]
    fn or_obj_2_test() {
        let json = json!({
            "key":{
             "id":1,
             "name":"a",
             "another":"d"
            }
        });
        let index = idx!(?filter!(
            filter!(op!(path!(@,path!("name"))), "==", op!("c")),
            ||,
            filter!(op!(path!(@,path!("another"))), "==", op!("d"))
        )
        );
        let chain = chain!(path!($), path!("key"), path!(index), path!("id"));
        let path_inst = json_path_instance(&chain, &json);
        let j1 = json!(1);
        assert_eq!(path_inst.find((&json).into()), json_path_value![&j1,])
    }

    #[test]
    fn and_arr_test() {
        let json = json!({
            "key":[
                {"city":"London","capital":true, "size": "big"},
                {"city":"Berlin","capital":true,"size": "big"},
                {"city":"Tokyo","capital":true,"size": "big"},
                {"city":"Moscow","capital":true,"size": "big"},
                {"city":"Athlon","capital":false,"size": "small"},
                {"city":"Dortmund","capital":false,"size": "big"},
                {"city":"Dublin","capital":true,"size": "small"},
            ]
        });
        let index = idx!(?filter!(
            filter!(op!(path!(@,path!("capital"))), "==", op!(false)),
            &&,
            filter!(op!(path!(@,path!("size"))), "==", op!("small"))
        )
        );
        let chain = chain!(path!($), path!("key"), path!(index), path!("city"));
        let path_inst = json_path_instance(&chain, &json);
        let a = json!("Athlon");
        assert_eq!(path_inst.find((&json).into()), json_path_value![&a,])
    }

    #[test]
    fn and_obj_test() {
        let json = json!({
            "key":{
             "id":1,
             "name":"a",
             "another":"b"
            }
        });
        let index = idx!(?filter!(
            filter!(op!(path!(@,path!("name"))), "==", op!("a")),
            &&,
            filter!(op!(path!(@,path!("another"))), "==", op!("b"))
        )
        );
        let chain = chain!(path!($), path!("key"), path!(index), path!("id"));
        let path_inst = json_path_instance(&chain, &json);
        let j1 = json!(1);
        assert_eq!(path_inst.find((&json).into()), json_path_value![&j1,])
    }

    #[test]
    fn and_obj_2_test() {
        let json = json!({
            "key":{
             "id":1,
             "name":"a",
             "another":"d"
            }
        });
        let index = idx!(?filter!(
            filter!(op!(path!(@,path!("name"))), "==", op!("c")),
            &&,
            filter!(op!(path!(@,path!("another"))), "==", op!("d"))
        )
        );
        let chain = chain!(path!($), path!("key"), path!(index), path!("id"));
        let path_inst = json_path_instance(&chain, &json);
        assert_eq!(path_inst.find((&json).into()), vec![NoValue])
    }
}
