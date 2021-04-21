use serde_json::Value;

#[derive(Debug, Clone)]
pub enum JsonPath {
    Root,
    Field(String),
    Chain(Vec<JsonPath>),
    Descent(String),
    Index(JsonPathIndex),
    Current(Box<JsonPath>),
    Wildcard,
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

#[derive(Debug, Clone)]
pub enum JsonPathIndex {
    Single(usize),
    Union(Vec<Operand>),
    Slice(i32, i32, usize),
    Filter(Operand, FilterSign, Operand),
}

#[derive(Debug, Clone)]
pub enum Operand {
    Static(Value),
    Dynamic(Box<JsonPath>),
}


#[derive(Debug, Clone,PartialEq)]
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

impl PartialEq for JsonPath {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (JsonPath::Root, JsonPath::Root) => true,
            (JsonPath::Descent(k1), JsonPath::Descent(k2)) => k1 == k2,
            (JsonPath::Field(k1), JsonPath::Field(k2)) => k1 == k2,
            (JsonPath::Wildcard, JsonPath::Wildcard) => true,
            (JsonPath::Empty, JsonPath::Empty) => true,
            (JsonPath::Current(jp1), JsonPath::Current(jp2)) => jp1 == jp2,
            (JsonPath::Chain(ch1), JsonPath::Chain(ch2)) => {
                if ch1.len() == ch2.len() {
                    ch1.iter()
                        .zip(ch2.iter())
                        .filter(|(a, b)| a != b).count() > 0
                } else { false }
            }
            (JsonPath::Index(idx1), JsonPath::Index(idx2)) => idx1 == idx2,
            (_, _) => false
        }
    }
}

impl PartialEq for JsonPathIndex {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (JsonPathIndex::Slice(s1, e1, st1), JsonPathIndex::Slice(s2, e2, st2)) =>
                s1 == s2 && e1 == e2 && st1 == st2,
            (JsonPathIndex::Single(el1), JsonPathIndex::Single(el2)) => el1 == el2,
            (JsonPathIndex::Union(elms1), JsonPathIndex::Union(elems2)) => {
                if elms1.len() == elems2.len() {
                    elms1.iter()
                        .zip(elems2.iter())
                        .filter(|(a, b)| a != b).count() > 0
                } else { false }
            }
            (JsonPathIndex::Filter(l1, s1, r1), JsonPathIndex::Filter(l2, s2, r2)) =>
                l1 == l2 && s1 == s2 && r1 == r2,
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

