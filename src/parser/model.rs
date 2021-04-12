use serde_json::Value;

#[derive(Debug,Clone)]
pub enum JsonPath<'a> {
    Root,
    Field(String),
    Chain(&'a Vec<&'a JsonPath<'a>>),
    Descent(String),
    Index(JsonPathIndex<'a>),
    Current(Option<&'a JsonPath<'a>>),
    Wildcard,
}


#[derive(Debug,Clone)]
pub enum JsonPathIndex<'a> {
    Single(usize),
    Union(Vec<&'a Operand<'a>>),
    Slice(i32, i32, usize),
    Filter(Operand<'a>, FilterSign, Operand<'a>),
}

#[derive(Debug,Clone)]
pub enum Operand<'a> {
    Static(Value),
    Dynamic(&'a JsonPath<'a>),
}

#[derive(Debug,Clone)]
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
    Exists,
}



