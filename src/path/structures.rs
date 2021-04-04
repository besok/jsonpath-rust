use serde_json::{Result, Value};

#[derive(Debug)]
pub enum JsonPath<'a> {
    Root,
    Field(String),
    Path(&'a Vec<&'a JsonPath<'a>>),
    Descent(String),
    Index(JsonPathIndex<'a>),
    Current(Option<&'a JsonPath<'a>>),
    Wildcard,
    Function(FnType)
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
pub enum FilterSign {
    Exist,
    Equal,
    Unequal,
    Less,
    Greater,
    LeOrEq,
    GrOrEq,
    Regex,
    In,
    Nin,
    Size,
    Empty,
    NoneOf,
    AnyOf,
    SubSetOf,
}


#[derive(Debug)]
pub enum FnType {
    Len
}

#[derive(Debug)]
pub enum ScriptSign {}


pub fn parse(json: &str) -> Result<Value> {
    serde_json::from_str(json)
}


#[cfg(test)]
mod tests {
    use crate::path::structures::parse;

    #[test]
    fn parser_dummy_test() {
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
