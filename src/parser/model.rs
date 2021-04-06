use serde_json::Value;

#[derive(Debug)]
pub enum JsonPath<'a> {
    Root,
    Field(String),
    Chain(&'a Vec<&'a JsonPath<'a>>),
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
    NoneOf,
    AnyOf,
    SubSetOf,
    Exists
}


#[derive(Debug)]
pub enum FnType {
    Len
}

#[derive(Debug)]
pub enum ScriptSign {}