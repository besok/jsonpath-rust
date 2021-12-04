use serde_json::Value;
use crate::parse_json_path;

/// The basic structures for parsing json paths.
/// The common logic of the structures pursues to correspond the internal parsing structure.
#[derive(Debug, Clone)]
pub enum JsonPath {
    /// The $ operator
    Root,
    /// Field represents key
    Field(String),
    /// The whole chain of the path.
    Chain(Vec<JsonPath>),
    /// The .. operator
    Descent(String),
    /// The indexes for array
    Index(JsonPathIndex),
    /// The @ operator
    Current(Box<JsonPath>),
    /// The * operator
    Wildcard,
    /// The item uses to define the unresolved state
    Empty,
}

impl JsonPath {
    pub fn descent(key: &str) -> Self {
        JsonPath::Descent(String::from(key))
    }
    pub fn field(key: &str) -> Self {
        JsonPath::Field(String::from(key))
    }
}

impl JsonPath {
    /// allows to create an JsonPath from string
    pub fn from_str(v: &str) -> Result<JsonPath, String> {
        parse_json_path(v).map_err(|e| e.to_string()).map_err(|e| e.to_string())
    }
}

#[derive(Debug, Clone)]
pub enum JsonPathIndex {
    /// The single element in array
    Single(Value),
    /// Union represents a several indexes
    UnionIndex(Vec<Value>),
    /// Union represents a several keys
    UnionKeys(Vec<String>),
    /// DEfault slice where the items are start/end/step respectively
    Slice(i32, i32, usize),
    /// Filter ?()
    Filter(Operand, FilterSign, Operand),
}

impl JsonPathIndex {
    pub fn exists(op: Operand) -> Self {
        JsonPathIndex::Filter(op, FilterSign::Exists, Operand::Dynamic(Box::new(JsonPath::Empty)))
    }
}

/// Operand for filtering expressions
#[derive(Debug, Clone)]
pub enum Operand {
    Static(Value),
    Dynamic(Box<JsonPath>),
}

impl Operand {
    pub fn str(v: &str) -> Self {
        Operand::Static(Value::from(v))
    }
    pub fn val(v: Value) -> Self { Operand::Static(v) }
    pub fn path(p: JsonPath) -> Self {
        Operand::Dynamic(Box::new(p))
    }
}

/// The operators for filtering functions
#[derive(Debug, Clone, PartialEq)]
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

impl FilterSign {
    pub fn new(key: &str) -> Self {
        match key {
            "==" => FilterSign::Equal,
            "!=" => FilterSign::Unequal,
            "<" => FilterSign::Less,
            ">" => FilterSign::Greater,
            "<=" => FilterSign::LeOrEq,
            ">=" => FilterSign::GrOrEq,
            "~=" => FilterSign::Regex,
            "in" => FilterSign::In,
            "nin" => FilterSign::Nin,
            "size" => FilterSign::Size,
            "noneOf" => FilterSign::NoneOf,
            "anyOf" => FilterSign::AnyOf,
            "subsetOf" => FilterSign::SubSetOf,
            _ => FilterSign::Exists,
        }
    }
}

impl PartialEq for JsonPath {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (JsonPath::Root, JsonPath::Root) => true,
            (JsonPath::Descent(k1), JsonPath::Descent(k2)) => k1 == k2,
            (JsonPath::Field(k1), JsonPath::Field(k2)) => k1 == k2,
            (JsonPath::Wildcard, JsonPath::Wildcard) => true,
            (JsonPath::Empty, JsonPath::Empty) => true,
            (JsonPath::Current(jp1), JsonPath::Current(jp2)) => jp1 == jp2,
            (JsonPath::Chain(ch1), JsonPath::Chain(ch2)) => ch1 == ch2,
            (JsonPath::Index(idx1), JsonPath::Index(idx2)) => idx1 == idx2,
            (_, _) => false
        }
    }
}

impl PartialEq for JsonPathIndex {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (JsonPathIndex::Slice(s1, e1, st1),
                JsonPathIndex::Slice(s2, e2, st2)) => s1 == s2 && e1 == e2 && st1 == st2,
            (JsonPathIndex::Single(el1), JsonPathIndex::Single(el2)) => el1 == el2,
            (JsonPathIndex::UnionIndex(elems1), JsonPathIndex::UnionIndex(elems2)) => elems1 == elems2,
            (JsonPathIndex::UnionKeys(elems1), JsonPathIndex::UnionKeys(elems2)) => elems1 == elems2,
            (JsonPathIndex::Filter(l1, s1, r1),
                JsonPathIndex::Filter(l2, s2, r2)) => l1 == l2 && s1 == s2 && r1 == r2,
            (_, _) => false
        }
    }
}

impl PartialEq for Operand {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Operand::Static(v1), Operand::Static(v2)) => v1 == v2,
            (Operand::Dynamic(jp1), Operand::Dynamic(jp2)) => jp1 == jp2,
            (_, _) => false
        }
    }
}

