use serde_json::{Result, Value};

enum JsonPath {
    Root,
    Current,
    Path(Vec<JsonPath>),
    Descent,
    Wildcard,
    Index(String, JsonPathIndex),
}

enum JsonPathIndex {
    Single(u32),
    Union(Vec<JsonPath>),
    Slice(i32, i32, i32),
    Filter(Operand, FilterSign, Operand),
    Script(Operand, ScriptSign, Operand),
}

enum Operand {
    Static(Value),
    Dynamic(JsonPath),
}

enum FilterSign {}

enum ScriptSign {}


fn parse(json: &str) -> Result<Value> {
    serde_json::from_str(json)
}






#[cfg(test)]
mod tests {
    use crate::structures::parse;

    #[test]
    fn it_works() {
        let res = parse(r#"
        {
            "name": "John Doe",
            "age": 43,
            "phones": [
                "+44 1234567",
                "+44 2345678"
            ]
        }"#).unwrap();

        for (el, v) in res.as_object().unwrap() {
            println!("{} : {}", el, v)
        }
    }
}
