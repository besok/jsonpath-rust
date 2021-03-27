use serde_json::{Value, Map};
use serde_json::json;
use serde_json::value::Value::Array;

trait Path<'a> {
    type Data;
    fn path(&self) -> Vec<&'a Self::Data>;
}

#[derive(Debug)]
struct ArraySlice<'a, T> {
    elements: &'a Vec<T>,
    start_index: i32,
    end_index: i32,
    step: usize,
}

impl<'a, T> ArraySlice<'a, T> {
    fn from(array: &'a Value,
            start_index: i32,
            end_index: i32,
            step: usize, ) -> Option<ArraySlice<'a, Value>> {
        array.as_array().map(|elems| ArraySlice::new(elems, start_index, end_index, step))
    }

    fn new(
        elements: &'a Vec<T>,
        start_index: i32,
        end_index: i32,
        step: usize,
    ) -> Self {
        ArraySlice { elements, start_index, end_index, step }
    }

    fn end(&self) -> Option<usize> {
        let len = self.elements.len() as i32;
        if self.end_index >= 0 {
            if self.end_index > len { None } else { Some(self.end_index as usize) }
        } else {
            if self.end_index < len * -1 { None } else { Some((len - (self.end_index * -1)) as usize) }
        }
    }

    fn start(&self) -> Option<usize> {
        let len = self.elements.len() as i32;
        if self.start_index >= 0 {
            if self.start_index > len { None } else { Some(self.start_index as usize) }
        } else {
            if self.start_index < len * -1 { None } else { Some((len - self.start_index * -1) as usize) }
        }
    }
}

impl<'a, T> Path<'a> for ArraySlice<'a, T> {
    type Data = T;

    fn path(&self) -> Vec<&'a Self::Data> {
        let mut filtered_elems: Vec<&Self::Data> = vec![];
        match (self.start(), self.end()) {
            (Some(start_idx), Some(end_idx)) => {
                for idx in (start_idx..end_idx).step_by(self.step) {
                    if let Some(v) = self.elements.get(idx) {
                        filtered_elems.push(v)
                    }
                }
                filtered_elems
            }
            _ => filtered_elems
        }
    }
}

struct ArrayIndex<'a, T> {
    elements: &'a Vec<T>,
    index: usize,
}

impl<'a, T> ArrayIndex<'a, T> {
    fn from(array: &'a Value, index: usize) -> Option<ArrayIndex<'a, Value>> {
        array.as_array().map(|elements| ArrayIndex::new(elements, index))
    }

    fn new(elements: &'a Vec<T>, index: usize) -> Self {
        ArrayIndex { elements, index }
    }
}

impl<'a, T> Path<'a> for ArrayIndex<'a, T> {
    type Data = T;

    fn path(&self) -> Vec<&'a Self::Data> {
        self.elements.get(self.index).map(|el| vec![el]).unwrap_or(vec![])
    }
}

struct ObjectField<'a> {
    fields: &'a Map<String, Value>,
    key: &'a String,
}

impl<'a> ObjectField<'a> {
    fn new(value: &'a Value, key: &'a String) -> Option<ObjectField<'a>> {
        value.as_object().map(|fields| ObjectField { fields, key })
    }
}

impl<'a> Path<'a> for ObjectField<'a> {
    type Data = Value;

    fn path(&self) -> Vec<&'a Self::Data> {
        self.fields.get(self.key).map(|el| vec![el]).unwrap_or(vec![])
    }
}

pub fn find_in_object<'a>(data: &'a Value, name: &String) -> &'a Value {
    if let Value::Object(map) = data {
        map.get(name).unwrap_or(&Value::Null)
    } else {
        &Value::Null
    }
}

#[cfg(test)]
mod tests {
    use crate::structures::{JsonPath, parse, JsonPathIndex};
    use crate::find::calculations::{ArraySlice, Path, ArrayIndex, ObjectField};
    use serde_json::Value;
    use serde_json::json;

    #[test]
    fn start_index() {
        let array = vec![0, 1, 2, 3, 4, 5];
        let mut slice = ArraySlice::new(&array, 0, 0, 0);

        assert_eq!(slice.start().unwrap(), 0);
        slice.start_index = 1;

        assert_eq!(slice.start().unwrap(), 1);

        slice.start_index = 2;
        assert_eq!(slice.start().unwrap(), 2);

        slice.start_index = 5;
        assert_eq!(slice.start().unwrap(), 5);

        slice.start_index = 7;
        assert_eq!(slice.start(), None);

        slice.start_index = -1;
        assert_eq!(slice.start().unwrap(), 5);

        slice.start_index = -5;
        assert_eq!(slice.start().unwrap(), 1);

        slice.end_index = 0;
        assert_eq!(slice.end().unwrap(), 0);

        slice.end_index = 5;
        assert_eq!(slice.end().unwrap(), 5);

        slice.end_index = -1;
        assert_eq!(slice.end().unwrap(), 5);

        slice.end_index = -5;
        assert_eq!(slice.end().unwrap(), 1);
    }

    #[test]
    fn slice_test() {
        let array = parse(r#"[0,1,2,3,4,5,6,7,8,9,10]"#).unwrap();

        let mut slice = ArraySlice::<Value>::from(&array, 0, 6, 2).unwrap();
        assert_eq!(slice.path(), vec![&json!(0), &json!(2), &json!(4)]);

        slice.step = 3;
        assert_eq!(slice.path(), vec![&json!(0), &json!(3)]);

        slice.start_index = -1;
        slice.end_index = 1;

        assert!(slice.path().is_empty());

        slice.start_index = -10;
        slice.end_index = 10;

        assert_eq!(slice.path(), vec![&json!(1), &json!(4), &json!(7)]);
    }

    #[test]
    fn index_test() {
        let array = parse(r#"[0,1,2,3,4,5,6,7,8,9,10]"#).unwrap();

        let mut index = ArrayIndex::<Value>::from(&array, 0).unwrap();

        assert_eq!(index.path(), vec![&json!(0)]);
        index.index = 10;
        assert_eq!(index.path(), vec![&json!(10)]);
        index.index = 100;
        assert!(index.path().is_empty());
    }

    #[test]
    fn object_test() {
        let res_income = parse(r#"
        {
            "product": {"key":42}
        }"#).unwrap();

        let key = String::from("product");
        let mut field = ObjectField::new(&res_income, &key).unwrap();
        assert_eq!(field.path(), vec![&json!({"key":42})]);

        let key = String::from("fake");

        field.key = &key;
        assert!(field.path().is_empty());
    }
}