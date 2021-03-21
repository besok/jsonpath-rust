use serde_json::Value;

pub fn find_in_object<'a>(data: &'a Value, name: &String) -> &'a Value {
    if let Value::Object(map) = data {
        map.get(name).unwrap_or(&Value::Null)
    } else {
        &Value::Null
    }
}

pub fn find_in_array(data: &Value, idx: usize) -> &Value {
    if let Value::Array(elems) = data {
        elems.get(idx).unwrap_or(&Value::Null)
    } else {
        &Value::Null
    }
}