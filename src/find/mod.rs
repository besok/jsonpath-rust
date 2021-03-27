mod calculations;

use serde_json::{Value};
use crate::structures::{JsonPath, JsonPathIndex};
use crate::find::calculations::{find_in_object};

pub trait Find {
    fn find<'a>(&self, data: &'a Value, root: &'a Value) -> &'a Value;
}


impl Find for JsonPathIndex<'_> {
    fn find<'a>(&self, data: &'a Value, root: &'a Value) -> &'a Value {
        match self {
            // JsonPathIndex::Single(idx) => find_in_array(data, *idx),
            // JsonPathIndex::Slice(start, end, step) => slice_in_array(data, *start, *end, *step),
            _ => &Value::Null
        }
    }
}

impl Find for JsonPath<'_> {
    fn find<'a>(&self, data: &'a Value, root: &'a Value) -> &'a Value {
        match self {
            JsonPath::Root => root,
            JsonPath::Path(elems) => elems.iter().fold(data, |n, p| { p.find(n, root) }),
            JsonPath::Field(key) => find_in_object(data, key),
            JsonPath::Index(key, idx) => idx.find(find_in_object(data, key), root),
            _ => root
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::structures::{JsonPath, parse, JsonPathIndex};
    use self::super::Find;

    #[test]
    fn dummy_test() {
        let res_income = parse(r#"
        {
            "name": "John Doe",
            "age": 43,
            "phones": [
                "+44 1234567",
                "+44 2345678"
            ]
        }"#).unwrap();
        let res = JsonPath::Root.find(&res_income, &res_income);
        assert_eq!(res, &res_income)
    }

    #[test]
    fn dummy_path_test() {
        let res_income = parse(r#"
        {
            "name": "John Doe",
            "age": 43,
            "phones": [
                "+44 1234567",
                "+44 2345678"
            ]
        }"#).unwrap();
        let res = JsonPath::Path(&vec![JsonPath::Root, JsonPath::Root]).find(&res_income, &res_income);
        assert_eq!(res, &res_income)
    }

    #[test]
    fn dummy_name_test() {
        let res_income = parse(r#"
        {
            "product": {
            "trait": {
              "detail":{"id":42}
            }}
        }"#).unwrap();
        let res_expected = parse("42").unwrap();
        let path = vec![
            JsonPath::Root,
            JsonPath::Field(String::from("product")),
            JsonPath::Field(String::from("trait")),
            JsonPath::Field(String::from("detail")),
            JsonPath::Field(String::from("id")),
        ];
        let res = JsonPath::Path(&path).find(&res_income, &res_income);
        assert_eq!(res, &res_expected)
    }

    #[test]
    fn dummy_index_in_array_test() {
        let res_income = parse(r#"
        {
            "product": {
            "trait": {
              "detail":{"id":[0,1,2,3,4,5,6,7,8,9,10]}
            }}
        }"#).unwrap();
        let res_expected = parse("5").unwrap();
        let path = vec![
            JsonPath::Root,
            JsonPath::Field(String::from("product")),
            JsonPath::Field(String::from("trait")),
            JsonPath::Field(String::from("detail")),
            JsonPath::Index(String::from("id"), JsonPathIndex::Single(5)),
        ];
        let res = JsonPath::Path(&path).find(&res_income, &res_income);
        assert_eq!(res, &res_expected)
    }
}