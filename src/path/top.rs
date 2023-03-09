use crate::parser::model::*;
use crate::path::{json_path_instance, JsonPathValue, Path, PathInstance};
use serde_json::value::Value::{Array, Object};
use serde_json::{json, Value};

/// to process the element [*]
pub(crate) struct Wildcard {}

impl<'a> Path<'a> for Wildcard {
    type Data = Value;

    fn find(&self, data: JsonPathValue<'a, Self::Data>) -> Vec<JsonPathValue<'a, Self::Data>> {
        data.map_slice(|data| match data {
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
            _ => vec![],
        })
    }
}

/// empty path. Returns incoming data.
pub(crate) struct IdentityPath {}

impl<'a> Path<'a> for IdentityPath {
    type Data = Value;

    fn find(&self, data: JsonPathValue<'a, Self::Data>) -> Vec<JsonPathValue<'a, Self::Data>> {
        vec![data]
    }
}

pub(crate) struct EmptyPath {}

impl<'a> Path<'a> for EmptyPath {
    type Data = Value;

    fn find(&self, _data: JsonPathValue<'a, Self::Data>) -> Vec<JsonPathValue<'a, Self::Data>> {
        vec![]
    }
}

/// process $ element
pub(crate) struct RootPointer<'a, T> {
    root: &'a T,
}

impl<'a, T> RootPointer<'a, T> {
    pub(crate) fn new(root: &'a T) -> RootPointer<'a, T> {
        RootPointer { root }
    }
}

impl<'a> Path<'a> for RootPointer<'a, Value> {
    type Data = Value;

    fn find(&self, _data: JsonPathValue<'a, Self::Data>) -> Vec<JsonPathValue<'a, Self::Data>> {
        vec![JsonPathValue::Slice(self.root)]
    }
}

/// process object fields like ['key'] or .key
pub(crate) struct ObjectField<'a> {
    key: &'a str,
}

impl<'a> ObjectField<'a> {
    pub(crate) fn new(key: &'a str) -> ObjectField<'a> {
        ObjectField { key }
    }
}

impl<'a> Clone for ObjectField<'a> {
    fn clone(&self) -> Self {
        ObjectField::new(self.key)
    }
}

impl<'a> Path<'a> for FnPath {
    type Data = Value;

    fn flat_find(
        &self,
        input: Vec<JsonPathValue<'a, Self::Data>>,
        is_search_length: bool,
    ) -> Vec<JsonPathValue<'a, Self::Data>> {
        let len = if is_search_length {
            json!(input.len())
        } else {
            match input.get(0) {
                None => json!(Value::Null),
                Some(v) => match v {
                    JsonPathValue::NewValue(Array(arr)) | JsonPathValue::Slice(Array(arr)) => {
                        json!(arr.len())
                    }
                    _ => json!(Value::Null),
                },
            }
        };

        vec![JsonPathValue::NewValue(len)]
    }

    fn needs_all(&self) -> bool {
        true
    }
}

pub(crate) enum FnPath {
    Size,
}

impl<'a> Path<'a> for ObjectField<'a> {
    type Data = Value;

    fn find(&self, data: JsonPathValue<'a, Self::Data>) -> Vec<JsonPathValue<'a, Self::Data>> {
        data.map_slice(|data| match data {
            Object(fields) => fields.get(self.key).map(|e| vec![e]).unwrap_or_default(),
            _ => vec![],
        })
    }
}
/// the top method of the processing ..=
pub(crate) struct DescentWildcard;

impl<'a> Path<'a> for DescentWildcard {
    type Data = Value;

    fn find(&self, data: JsonPathValue<'a, Self::Data>) -> Vec<JsonPathValue<'a, Self::Data>> {
        data.map_slice(deep_flatten)
    }
}

// todo rewrite to tail rec
fn deep_flatten(data: &Value) -> Vec<&Value> {
    let mut acc = vec![];
    match data {
        Object(elems) => {
            for v in elems.values() {
                acc.push(v);
                acc.append(&mut deep_flatten(v));
            }
        }
        Array(elems) => {
            for v in elems.iter() {
                acc.push(v);
                acc.append(&mut deep_flatten(v));
            }
        }
        _ => (),
    }
    acc
}

// todo rewrite to tail rec
fn deep_path_by_key<'a>(data: &'a Value, key: ObjectField<'a>) -> Vec<&'a Value> {
    let mut level: Vec<&Value> = JsonPathValue::slice_into_vec(key.find(data.into()));
    match data {
        Object(elems) => {
            let mut next_levels: Vec<&Value> = elems
                .values()
                .flat_map(|v| deep_path_by_key(v, key.clone()))
                .collect();
            level.append(&mut next_levels);
            level
        }
        Array(elems) => {
            let mut next_levels: Vec<&Value> = elems
                .iter()
                .flat_map(|v| deep_path_by_key(v, key.clone()))
                .collect();
            level.append(&mut next_levels);
            level
        }
        _ => level,
    }
}

/// processes decent object like ..
pub(crate) struct DescentObject<'a> {
    key: &'a str,
}

impl<'a> Path<'a> for DescentObject<'a> {
    type Data = Value;

    fn find(&self, data: JsonPathValue<'a, Self::Data>) -> Vec<JsonPathValue<'a, Self::Data>> {
        data.map_slice(|data| deep_path_by_key(data, ObjectField::new(self.key)))
    }
}

impl<'a> DescentObject<'a> {
    pub fn new(key: &'a str) -> Self {
        DescentObject { key }
    }
}

/// the top method of the processing representing the chain of other operators
pub(crate) struct Chain<'a> {
    chain: Vec<PathInstance<'a>>,
    is_search_length: bool,
}

impl<'a> Chain<'a> {
    pub fn new(chain: Vec<PathInstance<'a>>, is_search_length: bool) -> Self {
        Chain {
            chain,
            is_search_length,
        }
    }
    pub fn from(chain: &'a [JsonPath], root: &'a Value) -> Self {
        let chain_len = chain.len();
        let is_search_length = if chain_len > 2 {
            let mut res = false;
            // if the result of the slice ex[eccted to be a slice, union or filter -
            // length should return length of resulted array
            // In all other cases, including single index, we should fetch item from resulting array
            // and return length of that item
            res = match chain.get(chain_len - 1).unwrap() {
                JsonPath::Fn(Function::Length) => {
                    for item in chain.iter() {
                        match (item, res) {
                            // if we found union, slice, filter or wildcard - set search to true
                            (
                                JsonPath::Index(JsonPathIndex::UnionIndex(_))
                                | JsonPath::Index(JsonPathIndex::UnionKeys(_))
                                | JsonPath::Index(JsonPathIndex::Slice(_, _, _))
                                | JsonPath::Index(JsonPathIndex::Filter(_))
                                | JsonPath::Wildcard,
                                false,
                            ) => {
                                res = true;
                            }
                            // if we found a fetching of single index - reset search to false
                            (JsonPath::Index(JsonPathIndex::Single(_)), true) => {
                                res = false;
                            }
                            (_, _) => {}
                        }
                    }
                    res
                }
                _ => false,
            };
            res
        } else {
            false
        };

        Chain::new(
            chain.iter().map(|p| json_path_instance(p, root)).collect(),
            is_search_length,
        )
    }
}

impl<'a> Path<'a> for Chain<'a> {
    type Data = Value;

    fn find(&self, data: JsonPathValue<'a, Self::Data>) -> Vec<JsonPathValue<'a, Self::Data>> {
        let mut res = vec![data];

        for inst in self.chain.iter() {
            if inst.needs_all() {
                res = inst.flat_find(res, self.is_search_length)
            } else {
                res = res.into_iter().flat_map(|d| inst.find(d)).collect()
            }
        }
        res
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::model::{JsonPath, JsonPathIndex};
    use crate::path::top::{deep_flatten, json_path_instance, Function, ObjectField, RootPointer};
    use crate::path::{JsonPathValue, Path};
    use crate::{chain, function, idx, json_path_value, path};
    use serde_json::json;
    use serde_json::Value;

    #[test]
    fn object_test() {
        let js = json!({"product": {"key":42}});
        let res_income = json_path_value!(&js);

        let key = String::from("product");
        let mut field = ObjectField::new(&key);
        let js = json!({"key":42});
        assert_eq!(field.find(res_income.clone()), vec![json_path_value!(&js)]);

        let key = String::from("fake");
        field.key = &key;
        assert!(field.find(res_income).is_empty());
    }

    #[test]
    fn root_test() {
        let res_income = json!({"product": {"key":42}});

        let root = RootPointer::<Value>::new(&res_income);

        assert_eq!(
            root.find(json_path_value!(&res_income)),
            vec![json_path_value!(&res_income)]
        )
    }

    #[test]
    fn path_instance_test() {
        let json = json!({"v": {"k":{"f":42,"array":[0,1,2,3,4,5],"object":{"field1":"val1","field2":"val2"}}}});
        let field1 = path!("v");
        let field2 = path!("k");
        let field3 = path!("f");
        let field4 = path!("array");
        let field5 = path!("object");

        let path_inst = json_path_instance(&path!($), &json);
        assert_eq!(
            path_inst.find(json_path_value!(&json)),
            vec![json_path_value!(&json)]
        );

        let path_inst = json_path_instance(&field1, &json);
        let exp_json =
            json!({"k":{"f":42,"array":[0,1,2,3,4,5],"object":{"field1":"val1","field2":"val2"}}});
        assert_eq!(
            path_inst.find(json_path_value!(&json)),
            vec![json_path_value!(&exp_json)]
        );

        let chain = chain!(path!($), field1.clone(), field2.clone(), field3);

        let path_inst = json_path_instance(&chain, &json);
        let exp_json = json!(42);
        assert_eq!(
            path_inst.find(json_path_value!(&json)),
            vec![json_path_value!(&exp_json)]
        );

        let chain = chain!(
            path!($),
            field1.clone(),
            field2.clone(),
            field4.clone(),
            path!(idx!(3))
        );
        let path_inst = json_path_instance(&chain, &json);
        let exp_json = json!(3);
        assert_eq!(
            path_inst.find(json_path_value!(&json)),
            vec![json_path_value!(&exp_json)]
        );

        let index = idx!([1;-1;2]);
        let chain = chain!(
            path!($),
            field1.clone(),
            field2.clone(),
            field4.clone(),
            path!(index)
        );
        let path_inst = json_path_instance(&chain, &json);
        let one = json!(1);
        let tree = json!(3);
        assert_eq!(
            path_inst.find(json_path_value!(&json)),
            vec![json_path_value!(&one), json_path_value!(&tree)]
        );

        let union = idx!(idx 1,2 );
        let chain = chain!(
            path!($),
            field1.clone(),
            field2.clone(),
            field4,
            path!(union)
        );
        let path_inst = json_path_instance(&chain, &json);
        let tree = json!(1);
        let two = json!(2);
        assert_eq!(
            path_inst.find(json_path_value!(&json)),
            vec![json_path_value!(&tree), json_path_value!(&two)]
        );

        let union = idx!("field1", "field2");
        let chain = chain!(path!($), field1.clone(), field2, field5, path!(union));
        let path_inst = json_path_instance(&chain, &json);
        let one = json!("val1");
        let two = json!("val2");
        assert_eq!(
            path_inst.find(json_path_value!(&json)),
            vec![json_path_value!(&one), json_path_value!(&two)]
        );
    }
    #[test]
    fn path_descent_arr_test() {
        let json = json!([{"a":1}]);
        let chain = chain!(path!($), path!(.."a"));
        let path_inst = json_path_instance(&chain, &json);

        let one = json!(1);
        let expected_res = vec![json_path_value!(&one)];
        assert_eq!(path_inst.find(json_path_value!(&json)), expected_res)
    }
    #[test]
    fn deep_path_test() {
        let value = json!([1]);
        let r = deep_flatten(&value);
        assert_eq!(r, vec![&json!(1)])
    }

    #[test]
    fn path_descent_w_array_test() {
        let json = json!(
        {
            "key1": [1]
        });
        let chain = chain!(path!($), path!(..*));
        let path_inst = json_path_instance(&chain, &json);

        let arr = json!([1]);
        let one = json!(1);

        let expected_res = vec![json_path_value!(&arr), json_path_value!(&one)];
        assert_eq!(path_inst.find(json_path_value!(&json)), expected_res)
    }
    #[test]
    fn path_descent_w_nested_array_test() {
        let json = json!(
        {
            "key2" : [{"a":1},{}]
        });
        let chain = chain!(path!($), path!(..*));
        let path_inst = json_path_instance(&chain, &json);

        let arr2 = json!([{"a": 1},{}]);
        let obj = json!({"a": 1});
        let empty = json!({});

        let one = json!(1);

        let expected_res = vec![
            json_path_value!(&arr2),
            json_path_value!(&obj),
            json_path_value!(&one),
            json_path_value!(&empty),
        ];
        assert_eq!(path_inst.find(json_path_value!(&json)), expected_res)
    }

    #[test]
    fn path_descent_w_test() {
        let json = json!(
        {
            "key1": [1],
            "key2": "key",
            "key3": {
                "key1": "key1",
                "key2": {
                    "key1": {
                        "key1": 0
                         }
                     }
            }
        });
        let chain = chain!(path!($), path!(..*));
        let path_inst = json_path_instance(&chain, &json);

        let key1 = json!([1]);
        let one = json!(1);
        let zero = json!(0);
        let key = json!("key");
        let key1_s = json!("key1");

        let key_3 = json!(  {
          "key1": "key1",
          "key2": {
            "key1": {
              "key1": 0
            }
          }
        });
        let key_sec = json!(    {
          "key1": {
            "key1": 0
          }
        });
        let key_th = json!(  {
          "key1": 0
        });

        let expected_res = vec![
            json_path_value!(&key1),
            json_path_value!(&one),
            json_path_value!(&key),
            json_path_value!(&key_3),
            json_path_value!(&key1_s),
            json_path_value!(&key_sec),
            json_path_value!(&key_th),
            json_path_value!(&zero),
        ];
        assert_eq!(path_inst.find(json_path_value!(&json)), expected_res)
    }
    #[test]
    fn path_descent_test() {
        let json = json!(
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
        });
        let chain = chain!(path!($), path!(.."key1"));
        let path_inst = json_path_instance(&chain, &json);

        let res1 = json!([1, 2, 3]);
        let res2 = json!("key1");
        let res3 = json!({"key1":0});
        let res4 = json!(0);

        let expected_res = vec![
            json_path_value!(&res1),
            json_path_value!(&res2),
            json_path_value!(&res3),
            json_path_value!(&res4),
        ];
        assert_eq!(path_inst.find(json_path_value!(&json)), expected_res)
    }

    #[test]
    fn wildcard_test() {
        let json = json!({
            "key1": [1,2,3],
            "key2": "key",
            "key3": {}
        });

        let chain = chain!(path!($), path!(*));
        let path_inst = json_path_instance(&chain, &json);

        let res1 = json!([1, 2, 3]);
        let res2 = json!("key");
        let res3 = json!({});

        let expected_res = vec![
            json_path_value!(&res1),
            json_path_value!(&res2),
            json_path_value!(&res3),
        ];
        assert_eq!(path_inst.find(json_path_value!(&json)), expected_res)
    }

    #[test]
    fn length_test() {
        let json = json!({
            "key1": [1,2,3],
            "key2": "key",
            "key3": {}
        });

        let chain = chain!(path!($), path!(*), function!(length));
        let path_inst = json_path_instance(&chain, &json);

        assert_eq!(
            path_inst.flat_find(vec![json_path_value!(&json)], true),
            vec![json_path_value!(json!(3))]
        );

        let chain = chain!(path!($), path!("key1"), function!(length));
        let path_inst = json_path_instance(&chain, &json);
        assert_eq!(
            path_inst.flat_find(vec![json_path_value!(&json)], false),
            vec![json_path_value!(json!(3))]
        );
    }
}
