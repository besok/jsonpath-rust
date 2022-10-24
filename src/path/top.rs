use serde_json::{Value};
use serde_json::value::Value::{Array, Object};
use crate::path::{PathInstance, json_path_instance, Path, JsonPathValue};
use crate::parser::model::*;

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
            _ => vec![]
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


impl<'a> Path<'a> for ObjectField<'a> {
    type Data = Value;

    fn find(&self, data: JsonPathValue<'a, Self::Data>) -> Vec<JsonPathValue<'a, Self::Data>> {
        data.map_slice(|data|
            data.as_object()
                .and_then(|fileds| fileds.get(self.key))
                .map(|e| vec![e])
                .unwrap_or_default()
        )
    }
}

/// processes decent object like ..
pub(crate) struct DescentObjectField<'a> {
    key: &'a str,
}

impl<'a> Path<'a> for DescentObjectField<'a> {
    type Data = Value;

    fn find(&self, data: JsonPathValue<'a, Self::Data>) -> Vec<JsonPathValue<'a, Self::Data>> {
        data.map_slice(|data| {
            fn deep_path<'a>(data: &'a Value, key: ObjectField<'a>) -> Vec<&'a Value> {
                let mut level: Vec<&Value> = JsonPathValue::slice_into_vec(key.find(data.into()));
                match data {
                    Object(elems) => {
                        let mut next_levels: Vec<&Value> =
                            elems.values().flat_map(|v| deep_path(v, key.clone())).collect();
                        level.append(&mut next_levels);
                        level
                    }
                    Array(elems) => {
                        let mut next_levels: Vec<&Value> =
                            elems.iter().flat_map(|v| deep_path(v, key.clone())).collect();
                        level.append(&mut next_levels);
                        level
                    }
                    _ => level
                }
            }
            let key = ObjectField::new(self.key);
            deep_path(data, key)
        })
    }
}

impl<'a> DescentObjectField<'a> {
    pub fn new(key: &'a str) -> Self {
        DescentObjectField { key }
    }
}

/// the top method of the processing representing the chain of other operators
pub(crate) struct Chain<'a> {
    chain: Vec<PathInstance<'a>>,
}

impl<'a> Chain<'a> {
    pub fn new(chain: Vec<PathInstance<'a>>) -> Self {
        Chain { chain }
    }
    pub fn from(chain: &'a [JsonPath], root: &'a Value) -> Self {
        Chain::new(chain.iter().map(|p| json_path_instance(p, root)).collect())
    }
}


impl<'a> Path<'a> for Chain<'a> {
    type Data = Value;

    fn find(&self, data: JsonPathValue<'a, Self::Data>) -> Vec<JsonPathValue<'a, Self::Data>> {
        data.map_slice(|data|
            self.chain
                .iter()
                .fold(
                    vec![data],
                    |inter_res, path| {
                        JsonPathValue::slice_into_vec(inter_res
                            .into_iter()
                            .flat_map(|d| path.find(JsonPathValue::Slice(d)))
                            .collect())
                    })
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::path::top::{ObjectField, RootPointer, json_path_instance};
    use serde_json::Value;
    use serde_json::json;
    use crate::parser::model::{JsonPath, JsonPathIndex};
    use crate::{chain, json_path_value, idx, path};
    use crate::path::{Path, JsonPathValue};

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

        assert_eq!(root.find(json_path_value!(&res_income)), vec![json_path_value!(&res_income)])
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
        assert_eq!(path_inst.find(json_path_value!(& json)), vec![json_path_value!(& json)]);


        let path_inst = json_path_instance(&field1, &json);
        let exp_json = json!({"k":{"f":42,"array":[0,1,2,3,4,5],"object":{"field1":"val1","field2":"val2"}}});
        assert_eq!(path_inst.find(json_path_value!(& json)), vec![json_path_value!(& exp_json)]);


        let chain = chain!(path!($),field1.clone(), field2.clone(), field3.clone());

        let path_inst = json_path_instance(&chain, &json);
        let exp_json = json!(42);
        assert_eq!(path_inst.find(json_path_value!(& json)), vec![json_path_value!(& exp_json)]);

        let chain = chain!(path!($),field1.clone(), field2.clone(), field4.clone(), path!(idx!(3)));
        let path_inst = json_path_instance(&chain, &json);
        let exp_json = json!(3);
        assert_eq!(path_inst.find(json_path_value!(&json)), vec![json_path_value!(&exp_json)]);

        let index = idx!([1;-1;2]);
        let chain = chain!(path!($), field1.clone(), field2.clone(), field4.clone(), path!(index));
        let path_inst = json_path_instance(&chain, &json);
        let one = json!(1);
        let tree = json!(3);
        assert_eq!(path_inst.find(json_path_value!(&json)), vec![json_path_value!(&one), json_path_value!(&tree)]);


        let union = idx!(idx 1,2 );
        let chain = chain!(path!($), field1.clone(), field2.clone(), field4.clone(), path!(union));
        let path_inst = json_path_instance(&chain, &json);
        let tree = json!(1);
        let two = json!(2);
        assert_eq!(path_inst.find(json_path_value!(&json)), vec![json_path_value!(&tree), json_path_value!(&two)]);

        let union = idx!("field1","field2");
        let chain = chain!(path!($), field1.clone(), field2.clone(), field5.clone(), path!(union));
        let path_inst = json_path_instance(&chain, &json);
        let one = json!("val1");
        let two = json!("val2");
        assert_eq!(path_inst.find(json_path_value!(&json)), vec![json_path_value!(&one), json_path_value!(&two)]);
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

        let res1 = json!([1,2,3]);
        let res2 = json!("key1");
        let res3 = json!({"key1":0});
        let res4 = json!(0);

        let expected_res = vec![json_path_value!(&res1), json_path_value!(&res2), json_path_value!(&res3), json_path_value!(&res4)];
        assert_eq!(path_inst.find(json_path_value!(&json)), expected_res)
    }

    #[test]
    fn wildcard_test() {
        let json =
            json!({
            "key1": [1,2,3],
            "key2": "key",
            "key3": {}
        });

        let chain = chain!(path!($),path!(*));
        let path_inst = json_path_instance(&chain, &json);

        let res1 = json!([1,2,3]);
        let res2 = json!("key");
        let res3 = json!({});

        let expected_res = vec![json_path_value!(&res1), json_path_value!(&res2), json_path_value!(&res3)];
        assert_eq!(path_inst.find(json_path_value!(&json)), expected_res)
    }
}