use serde_json::{Value, Map};
use serde_json::json;
use serde_json::value::Value::{Array, Object};
use crate::path::structures::{JsonPath, JsonPathIndex, FilterSign, Operand};
use std::fs::File;

pub(crate) trait Path<'a> {
    type Data;
    fn path(&self, data: &'a Self::Data) -> Vec<&'a Self::Data>;
}

type PathInstance<'a> = Box<dyn Path<'a, Data=Value> + 'a>;

fn process_path<'a>(json_path: &'a JsonPath, root: &'a Value) -> PathInstance<'a> {
    match json_path {
        JsonPath::Root => Box::new(RootPointer::new(root)),
        JsonPath::Field(key) => Box::new(ObjectField::new(key)),
        JsonPath::Path(chain) => Box::new(Chain::from(chain, root)),
        JsonPath::Wildcard => Box::new(Wildcard {}),
        JsonPath::Descent(key) => Box::new(DescentObjectField::new(key)),
        JsonPath::Current(Some(tail)) => Box::new(Current::new(process_path(tail, root))),
        JsonPath::Current(None) => Box::new(Current::none()),
        JsonPath::Index(index) => process_index(index, root),
        _ => Box::new(EmptyPath {})
    }
}

fn process_index<'a>(json_path_index: &'a JsonPathIndex, root: &'a Value) -> PathInstance<'a> {
    match json_path_index {
        JsonPathIndex::Single(index) => Box::new(ArrayIndex::new(*index)),
        JsonPathIndex::Slice(s, e, step) => Box::new(ArraySlice::new(*s, *e, *step)),
        JsonPathIndex::Union(elems) => Box::new(UnionIndex::from(elems, root)),
        JsonPathIndex::Filter(l, op, r) => Box::new(Filter::new(l, r, op, root)),
        _ => Box::new(EmptyPath {})
    }
}

fn process_operand<'a>(op: &'a Operand, root: &'a Value) -> PathInstance<'a> {
    match op {
        Operand::Static(v) => process_path(&JsonPath::Root, v),
        Operand::Dynamic(jp) => process_path(jp, root)
    }
}

pub(crate) struct Current<'a> {
    tail: Option<PathInstance<'a>>
}

impl<'a> Current<'a> {
    fn new(tail: PathInstance<'a>) -> Self {
        Current { tail: Some(tail) }
    }
    fn none() -> Self {
        Current { tail: None }
    }
}

impl<'a> Path<'a> for Current<'a> {
    type Data = Value;

    fn path(&self, data: &'a Self::Data) -> Vec<&'a Value> {
        let mut res: Vec<&'a Value> = vec![];

        match data {
            Array(elems) => {
                for el in elems {
                    let mut path = self.tail.as_ref().map(|p| p.path(el)).unwrap_or(vec![el]);
                    res.append(&mut path)
                }
                res
            }
            Object(elems) => {
                for el in elems.values() {
                    let mut path = self.tail.as_ref().map(|p| p.path(el)).unwrap_or(vec![el]);
                    res.append(&mut path)
                }
                res
            }
            _ => vec![]
        }
    }
}


pub(crate) struct Wildcard {}

impl<'a> Path<'a> for Wildcard {
    type Data = Value;

    fn path(&self, data: &'a Self::Data) -> Vec<&'a Self::Data> {
        match data {
            Array(elems) => {
                let mut res: Vec<&Value> = vec![];
                for el in elems.iter() {
                    res.push(el);
                }
                res
            }
            Object(elems) => {
                let mut res: Vec<&Value> = vec![];
                for el in elems.values() {
                    res.push(el);
                }
                res
            }
            _ => vec![]
        }
    }
}

pub(crate) struct EmptyPath {}

impl<'a> Path<'a> for EmptyPath {
    type Data = Value;
    fn path(&self, data: &'a Self::Data) -> Vec<&'a Self::Data> {
        vec![&data]
    }
}

pub(crate) struct RootPointer<'a, T> {
    root: &'a T
}

impl<'a, T> RootPointer<'a, T> {
    pub(crate) fn new(root: &'a T) -> RootPointer<'a, T> {
        RootPointer { root }
    }
}

impl<'a> Path<'a> for RootPointer<'a, Value> {
    type Data = Value;

    fn path(&self, _data: &'a Self::Data) -> Vec<&'a Self::Data> {
        vec![self.root]
    }
}

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
        } else {
            if self.end_index < len * -1 { None } else { Some((len - (self.end_index * -1)) as usize) }
        }
    }

    fn start(&self, len: i32) -> Option<usize> {
        if self.start_index >= 0 {
            if self.start_index > len { None } else { Some(self.start_index as usize) }
        } else {
            if self.start_index < len * -1 { None } else { Some((len - self.start_index * -1) as usize) }
        }
    }

    fn process<'a, T>(&self, elements: &'a Vec<T>) -> Vec<&'a T> {
        let len = elements.len() as i32;
        let mut filtered_elems: Vec<&T> = vec![];
        match (self.start(len), self.end(len)) {
            (Some(start_idx), Some(end_idx)) => {
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

    fn path(&self, data: &'a Self::Data) -> Vec<&'a Self::Data> {
        data.as_array()
            .map(|elems| self.process(elems))
            .unwrap_or(vec![])
    }
}

pub(crate) struct ArrayIndex {
    index: usize
}

impl ArrayIndex {
    pub(crate) fn new(index: usize) -> Self {
        ArrayIndex { index }
    }
}

impl<'a> Path<'a> for ArrayIndex {
    type Data = Value;

    fn path(&self, data: &'a Self::Data) -> Vec<&'a Self::Data> {
        data.as_array()
            .and_then(|elems| elems.get(self.index))
            .map(|e| vec![e])
            .unwrap_or(vec![])
    }
}

pub(crate) struct ObjectField<'a> {
    key: &'a String,
}

impl<'a> ObjectField<'a> {
    pub(crate) fn new(key: &'a String) -> ObjectField<'a> {
        ObjectField { key }
    }
}

impl<'a> Clone for ObjectField<'a> {
    fn clone(&self) -> Self {
        ObjectField::new(self.key)
    }
}

impl<'a> Path<'a> for ObjectField<'a> {
    type Data = Value;

    fn path(&self, data: &'a Self::Data) -> Vec<&'a Self::Data> {
        data.as_object()
            .and_then(|fileds| fileds.get(self.key))
            .map(|e| vec![e])
            .unwrap_or(vec![])
    }
}

pub(crate) struct DescentObjectField<'a> {
    key: &'a String,
}

impl<'a> Path<'a> for DescentObjectField<'a> {
    type Data = Value;

    fn path(&self, data: &'a Self::Data) -> Vec<&'a Self::Data> {
        fn deep_path<'a>(data: &'a Value, key: ObjectField<'a>) -> Vec<&'a Value> {
            let mut level: Vec<&Value> = key.path(data);
            match data.as_object() {
                Some(elems) => {
                    let mut next_levels: Vec<&Value> = elems.values().flat_map(|v| deep_path(v, key.clone())).collect();
                    level.append(&mut next_levels);
                    level
                }
                None => vec![]
            }
        }
        let key = ObjectField::new(self.key);
        deep_path(data, key)
    }
}

impl<'a> DescentObjectField<'a> {
    pub fn new(key: &'a String) -> Self {
        DescentObjectField { key }
    }
}


struct Chain<'a> {
    chain: Vec<PathInstance<'a>>,
}

impl<'a> Chain<'a> {
    pub fn new(chain: Vec<PathInstance<'a>>) -> Self {
        Chain { chain }
    }
    pub fn from_index(key: PathInstance<'a>, index: PathInstance<'a>) -> Self {
        Chain::new(vec![key, index])
    }
    pub fn from(chain: &'a Vec<&'a JsonPath>, root: &'a Value) -> Self {
        Chain::new(chain.iter().map(|p| process_path(p, root)).collect())
    }
}

impl<'a> Path<'a> for Chain<'a> {
    type Data = Value;

    fn path(&self, data: &'a Self::Data) -> Vec<&'a Self::Data> {
        self.chain.iter().fold(vec![data], |inter_res, path| {
            inter_res.iter().flat_map(|d| path.path(d)).collect()
        })
    }
}

struct UnionIndex<'a> {
    indexes: Vec<PathInstance<'a>>
}

impl<'a> UnionIndex<'a> {
    pub fn from(elems: &'a Vec<&'a JsonPath<'a>>, root: &'a Value) -> Self {
        let mut indexes: Vec<PathInstance<'a>> = vec![];

        for p in elems.iter() {
            indexes.push(match p {
                path @ JsonPath::Field(_) => process_path(path, root),
                JsonPath::Index(index @ JsonPathIndex::Single(_)) => process_index(&index, root),
                _ => Box::new(EmptyPath {})
            })
        }

        UnionIndex::new(indexes)
    }

    pub fn new(indexes: Vec<PathInstance<'a>>) -> Self {
        UnionIndex { indexes }
    }
}

impl<'a> Path<'a> for UnionIndex<'a> {
    type Data = Value;

    fn path(&self, data: &'a Self::Data) -> Vec<&'a Self::Data> {
        self.indexes.iter().flat_map(|e| e.path(data)).collect()
    }
}

struct Filter<'a> {
    left: PathInstance<'a>,
    right: PathInstance<'a>,
    op: &'a FilterSign,
}

impl<'a> Filter<'a> {
    fn new(left: &'a Operand, right: &'a Operand, op: &'a FilterSign, root: &'a Value) -> Self {
        Filter {
            left: process_operand(left, root),
            right: process_operand(right, root),
            op,
        }
    }

    fn process(op: &'a FilterSign, left: Vec<&'a Value>, right: Vec<&'a Value>) -> bool {
        match op {
            FilterSign::Equal => {}
            FilterSign::Unequal => !Filter::process(&FilterSign::Equal, left, right),
            FilterSign::Less => {}
            FilterSign::LeOrEq =>
                Filter::process(&FilterSign::Less, left, right)
                    || Filter::process(&FilterSign::Equal, left.clone(), right.clone()),
            FilterSign::Greater => !Filter::process(&FilterSign::LeOrEq, left, right),
            FilterSign::GrOrEq => !Filter::process(&FilterSign::Less, left, right),
            FilterSign::Regex => {}
            FilterSign::In => {}
            FilterSign::Nin => !Filter::process(&FilterSign::In, left, right),
            FilterSign::Size => {}
            FilterSign::Empty => {}
            FilterSign::NoneOf => {}
            FilterSign::AnyOf => {}
            FilterSign::SubSetOf => {}
            FilterSign::Exist => !left.is_empty()
        }
        false
    }
}

impl<'a> Path<'a> for Filter<'a> {
    type Data = Value;

    fn path(&self, data: &'a Self::Data) -> Vec<&'a Self::Data> {
        let mut res: Vec<&Value> = vec![];

        match data {
            Array(elems) => {
                for el in elems.iter() {
                    if Filter::process(&self.op, self.left.path(el), self.right.path(el)) {
                        res.push(el)
                    }
                }
                res
            }
            Object(pairs) => {
                for el in pairs.values() {
                    if Filter::process(&self.op, self.left.path(el), self.right.path(el)) {
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
    use crate::path::structures::{JsonPath, parse, JsonPathIndex};
    use crate::path::path::{ArraySlice, Path, ArrayIndex, ObjectField, RootPointer, process_path};
    use serde_json::Value;
    use serde_json::json;

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
        let array = parse(r#"[0,1,2,3,4,5,6,7,8,9,10]"#).unwrap();

        let mut slice = ArraySlice::new(0, 6, 2);
        assert_eq!(slice.path(&array), vec![&json!(0), &json!(2), &json!(4)]);

        slice.step = 3;
        assert_eq!(slice.path(&array), vec![&json!(0), &json!(3)]);

        slice.start_index = -1;
        slice.end_index = 1;

        assert!(slice.path(&array).is_empty());

        slice.start_index = -10;
        slice.end_index = 10;

        assert_eq!(slice.path(&array), vec![&json!(1), &json!(4), &json!(7)]);
    }

    #[test]
    fn index_test() {
        let array = parse(r#"[0,1,2,3,4,5,6,7,8,9,10]"#).unwrap();

        let mut index = ArrayIndex::new(0);

        assert_eq!(index.path(&array), vec![&json!(0)]);
        index.index = 10;
        assert_eq!(index.path(&array), vec![&json!(10)]);
        index.index = 100;
        assert!(index.path(&array).is_empty());
    }

    #[test]
    fn object_test() {
        let res_income = parse(r#"{"product": {"key":42}}"#).unwrap();

        let key = String::from("product");
        let mut field = ObjectField::new(&key);
        assert_eq!(field.path(&res_income), vec![&json!({"key":42})]);

        let key = String::from("fake");

        field.key = &key;
        assert!(field.path(&res_income).is_empty());
    }

    #[test]
    fn root_test() {
        let res_income = parse(r#"{"product": {"key":42}}"#).unwrap();

        let root = RootPointer::<Value>::new(&res_income);

        assert_eq!(root.path(&res_income), vec![&res_income])
    }

    #[test]
    fn path_instance_test() {
        let json = parse(r#"{"v": {"k":{"f":42,"array":[0,1,2,3,4,5],"object":{"field1":"val1","field2":"val2"}}}}"#).unwrap();
        let field1 = JsonPath::Field(String::from("v"));
        let field2 = JsonPath::Field(String::from("k"));
        let field3 = JsonPath::Field(String::from("f"));
        let field4 = JsonPath::Field(String::from("array"));
        let field5 = JsonPath::Field(String::from("object"));
        let field6 = JsonPath::Field(String::from("field1"));
        let field7 = JsonPath::Field(String::from("field2"));

        let root = JsonPath::Root;
        let path_inst = process_path(&root, &json);
        assert_eq!(path_inst.path(&json), vec![&json]);


        let path_inst = process_path(&field1, &json);
        let exp_json = parse(r#"{"k":{"f":42,"array":[0,1,2,3,4,5],"object":{"field1":"val1","field2":"val2"}}}"#).unwrap();
        assert_eq!(path_inst.path(&json), vec![&exp_json]);


        let chain = vec![&root, &field1, &field2, &field3];
        let chain = JsonPath::Path(&chain);

        let path_inst = process_path(&chain, &json);
        let exp_json = parse(r#"42"#).unwrap();
        assert_eq!(path_inst.path(&json), vec![&exp_json]);


        let index1 = JsonPath::Index(JsonPathIndex::Single(3));
        let index2 = JsonPath::Index(JsonPathIndex::Single(2));
        let chain = vec![&root, &field1, &field2, &field4, &index1];
        let chain = JsonPath::Path(&chain);
        let path_inst = process_path(&chain, &json);
        let exp_json = json!(3);
        assert_eq!(path_inst.path(&json), vec![&exp_json]);

        let index = JsonPath::Index(JsonPathIndex::Slice(1, -1, 2));
        let chain = vec![&root, &field1, &field2, &field4, &index];
        let chain = JsonPath::Path(&chain);
        let path_inst = process_path(&chain, &json);
        let one = json!(1);
        let tree = json!(3);
        assert_eq!(path_inst.path(&json), vec![&one, &tree]);


        let union = JsonPath::Index(JsonPathIndex::Union(vec![&index1, &index2]));
        let chain = vec![&root, &field1, &field2, &field4, &union];
        let chain = JsonPath::Path(&chain);
        let path_inst = process_path(&chain, &json);
        let tree = json!(3);
        let two = json!(2);
        assert_eq!(path_inst.path(&json), vec![&tree, &two]);

        let union = JsonPath::Index(JsonPathIndex::Union(vec![&field6, &field7]));
        let chain = vec![&root, &field1, &field2, &field5, &union];
        let chain = JsonPath::Path(&chain);
        let path_inst = process_path(&chain, &json);
        let one = json!("val1");
        let two = json!("val2");
        assert_eq!(path_inst.path(&json), vec![&one, &two]);
    }

    #[test]
    fn path_descent_test() {
        let json = parse(r#"
        {
            "key1": [1,2,3],
            "key2": "key",
            "key3": {
                "key1": "key1",
                "key2": {
                    "key1": {
                        "key1": 0
                         }
                     }
            }
        }"#).unwrap();
        let key = JsonPath::Descent(String::from("key1"));
        let root = JsonPath::Root;
        let chain = vec![&root, &key];
        let chain = JsonPath::Path(&chain);

        let path_inst = process_path(&chain, &json);

        let res1 = json!([1,2,3]);
        let res2 = json!("key1");
        let res3 = json!({"key1":0});
        let res4 = json!(0);

        let expected_res = vec![&res1, &res2, &res3, &res4];
        assert_eq!(path_inst.path(&json), expected_res)
    }

    #[test]
    fn wildcard_test() {
        let json = parse(r#"
        {
            "key1": [1,2,3],
            "key2": "key",
            "key3": {}
        }"#).unwrap();

        let root = JsonPath::Root;
        let wildcard = JsonPath::Wildcard;
        let chain = vec![&root, &wildcard];
        let chain = JsonPath::Path(&chain);
        let path_inst = process_path(&chain, &json);

        let res1 = json!([1,2,3]);
        let res2 = json!("key");
        let res3 = json!({});

        let expected_res = vec![&res1, &res2, &res3];
        assert_eq!(path_inst.path(&json), expected_res)
    }

    #[test]
    fn current_test() {
        let json = parse(r#"
        {
            "object":{
                "field_1":[1,2,3],
                "field_2":42,
                "field_3":{"a":"b"}

            }
        }"#).unwrap();

        let root = JsonPath::Root;
        let object = JsonPath::Field(String::from("object"));
        let cur = JsonPath::Current(None);

        let chain = vec![&root, &object, &cur];
        let chain = JsonPath::Path(&chain);

        let path_inst = process_path(&chain, &json);
        let res1 = json!([1,2,3]);
        let res2 = json!(42);
        let res3 = json!({"a":"b"});

        let expected_res = vec![&res1, &res2, &res3];
        assert_eq!(path_inst.path(&json), expected_res);

        let field_a = JsonPath::Field(String::from("a"));
        let chain_in = vec![&field_a];
        let chain_in = JsonPath::Path(&chain_in);
        let cur = JsonPath::Current(Some(&chain_in));

        let chain = vec![&root, &object, &cur];
        let chain = JsonPath::Path(&chain);

        let path_inst = process_path(&chain, &json);
        let res1 = json!("b");

        let expected_res = vec![&res1];
        assert_eq!(path_inst.path(&json), expected_res);
    }
}