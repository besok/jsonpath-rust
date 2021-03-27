use serde_json::{Result, Value};

#[derive(Debug)]
pub enum JsonPath<'a> {
    Root,
    Field(String),
    Path(&'a Vec<JsonPath<'a>>),
    Current,
    Descent,
    Wildcard,
    Index(String, JsonPathIndex<'a>),
}

#[derive(Debug)]
pub enum JsonPathIndex<'a> {
    Single(usize),
    Union(Vec<&'a JsonPath<'a>>),
    Slice(i32, i32, usize),
    Filter(Operand<'a>, FilterSign, Operand<'a>),
    Script(Operand<'a>, ScriptSign, Operand<'a>),
}

#[derive(Debug)]
pub enum Operand<'a> {
    Static(Value),
    Dynamic(&'a JsonPath<'a>),
}

#[derive(Debug)]
pub enum FilterSign {}

#[derive(Debug)]
pub enum ScriptSign {}


pub fn parse(json: &str) -> Result<Value> {
    serde_json::from_str(json)
}


#[cfg(test)]
mod tests {
    use crate::structures::parse;

    #[test]
    fn parser_check() {
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
