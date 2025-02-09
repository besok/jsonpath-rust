use std::fmt::{Display, Formatter};

/// Represents a JSONPath query with a list of segments.
#[derive(Debug, Clone)]
struct JpQuery  {
    segments: Vec<Segment>
}

impl Display for JpQuery {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "${}", self.segments.iter().map(|s| s.to_string()).collect::<String>())
    }
}
/// Enum representing different types of segments in a JSONPath query.
#[derive(Debug, Clone)]
enum Segment {
    /// Represents a descendant segment.
    Descendant,
    /// Represents a selector segment.
    Selector(Selector),
    /// Represents multiple selectors.
    Selectors(Vec<Selector>),
}

impl Display for Segment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Segment::Descendant => write!(f, ".."),
            Segment::Selector(selector) => write!(f, "{}", selector),
            Segment::Selectors(selectors) => write!(f, "{}", selectors.iter().map(|s| s.to_string()).collect::<String>()),
        }
    }
}
/// Enum representing different types of selectors in a JSONPath query.
#[derive(Debug, Clone)]
enum Selector {
    /// Represents a name selector.
    Name(String),
    /// Represents a wildcard selector.
    Wildcard,
    /// Represents an index selector.
    Index(i64),
    /// Represents a slice selector.
    Slice(i64, i64, i64),
    /// Represents a filter selector.
    Filter(Filter),
}

impl Display for Selector {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Selector::Name(name) => write!(f, "{}", name),
            Selector::Wildcard => write!(f, "*"),
            Selector::Index(index) => write!(f, "{}", index),
            Selector::Slice(start, end, step) => write!(f, "{}:{}:{}", start, end, step),
            Selector::Filter(filter) => write!(f, "[?{}]", filter),
        }
    }
}
/// Enum representing different types of filters in a JSONPath query.
#[derive(Debug, Clone)]
enum Filter {
    /// Represents a logical OR filter.
    Or(Box<Filter>, Box<Filter>),
    /// Represents a logical AND filter.
    And(Box<Filter>, Box<Filter>),
    /// Represents a logical NOT filter.
    Not(Box<Filter>),
    /// Represents an atomic filter.
    Atom(FilterAtom),
}

impl Display for Filter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Filter::Or(left, right) => write!(f, "{} || {}", left, right),
            Filter::And(left, right) => write!(f, "{} && {}", left, right),
            Filter::Not(expr) => write!(f, "!{}", expr),
            Filter::Atom(atom) => write!(f, "{}", atom),
        }
    }
}

/// Enum representing different types of atomic filters in a JSONPath query.
#[derive(Debug, Clone)]
enum FilterAtom {
    /// Represents a nested filter with an optional NOT flag.
    Filter {
        expr: Box<Filter>,
        not: bool,
    },
    /// Represents a test filter with an optional NOT flag.
    Test {
        expr: Box<Test>,
        not: bool,
    },
    /// Represents a comparison filter.
    Comparison(Box<Comparison>),
}

impl Display for FilterAtom {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FilterAtom::Filter { expr, not } => {
                if *not {
                    write!(f, "!{}", expr)
                } else {
                    write!(f, "{}", expr)
                }
            }
            FilterAtom::Test { expr, not } => {
                if *not {
                    write!(f, "!{}", expr)
                } else {
                    write!(f, "{}", expr)
                }
            }
            FilterAtom::Comparison(cmp) => write!(f, "{}", cmp),
        }
    }
}
/// Enum representing different types of comparisons in a JSONPath query.
#[derive(Debug, Clone)]
enum Comparison {
    /// Represents an equality comparison.
    Eq(Comparable, Comparable),
    /// Represents a non-equality comparison.
    Ne(Comparable, Comparable),
    /// Represents a greater-than comparison.
    Gt(Comparable, Comparable),
    /// Represents a greater-than-or-equal-to comparison.
    Gte(Comparable, Comparable),
    /// Represents a less-than comparison.
    Lt(Comparable, Comparable),
    /// Represents a less-than-or-equal-to comparison.
    Lte(Comparable, Comparable),
}

impl Display for Comparison {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Comparison::Eq(left, right) => write!(f, "{} == {}", left, right),
            Comparison::Ne(left, right) => write!(f, "{} != {}", left, right),
            Comparison::Gt(left, right) => write!(f, "{} > {}", left, right),
            Comparison::Gte(left, right) => write!(f, "{} >= {}", left, right),
            Comparison::Lt(left, right) => write!(f, "{} < {}", left, right),
            Comparison::Lte(left, right) => write!(f, "{} <= {}", left, right),
        }
    }
}

/// Enum representing different types of comparable values in a JSONPath query.
#[derive(Debug, Clone)]
enum Comparable {
    /// Represents a literal value.
    Literal(Literal),
    /// Represents a function.
    Function(TestFunction),
    /// Represents a singular query.
    SingularQuery(SingularQuery),
}

impl Display for Comparable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Comparable::Literal(literal) => write!(f, "{}", literal),
            Comparable::Function(func) => write!(f, "{}", func),
            Comparable::SingularQuery(query) => write!(f, "{}", query),
        }
    }
}

/// Enum representing different types of singular queries in a JSONPath query.
#[derive(Debug, Clone)]
enum SingularQuery {
    /// Represents a current node query.
    Current(Vec<SingularQuerySegment>),
    /// Represents a root node query.
    Root(Vec<SingularQuerySegment>),
}

impl Display for SingularQuery {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SingularQuery::Current(segments) => write!(f, "@.{}", segments.iter().map(|s| s.to_string()).collect::<String>()),
            SingularQuery::Root(segments) => write!(f, "$.{}", segments.iter().map(|s| s.to_string()).collect::<String>()),
        }
    }
}

/// Enum representing different types of singular query segments in a JSONPath query.
#[derive(Debug, Clone)]
enum SingularQuerySegment {
    /// Represents an index segment.
    Index(i64),
    /// Represents a name segment.
    Name(String),
}

impl Display for SingularQuerySegment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SingularQuerySegment::Index(index) => write!(f, "{}", index),
            SingularQuerySegment::Name(name) => write!(f, "{}", name),
        }
    }
}

/// Enum representing different types of tests in a JSONPath query.
#[derive(Debug, Clone)]
enum Test {
    /// Represents a relative query.
    RelQuery(Vec<Segment>),
    /// Represents an absolute query.
    AbsQuery(JpQuery),
    /// Represents a function test.
    Function(Box<TestFunction>),
}

impl Display for Test {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Test::RelQuery(segments) => write!(f, "{}", segments.iter().map(|s| s.to_string()).collect::<String>()),
            Test::AbsQuery(query) => write!(f, "{}", query),
            Test::Function(func) => write!(f, "{}", func),
        }
    }
}

/// Enum representing different types of test functions in a JSONPath query.
#[derive(Debug, Clone)]
enum TestFunction {
    /// Represents a custom function.
    Custom(String, Vec<FnArg>),
    /// Represents a length function.
    Length(Box<FnArg>),
    /// Represents a value function.
    Value(FnArg),
    /// Represents a count function.
    Count(FnArg),
    /// Represents a search function.
    Search(FnArg, FnArg),
    /// Represents a match function.
    Match(FnArg, FnArg),
}

impl Display for TestFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TestFunction::Custom(name, args) => write!(f, "{}({})", name, args.iter().map(|a| a.to_string()).collect::<String>()),
            TestFunction::Length(arg) => write!(f, "length({})", arg),
            TestFunction::Value(arg) => write!(f, "value({})", arg),
            TestFunction::Count(arg) => write!(f, "count({})", arg),
            TestFunction::Search(arg1, arg2) => write!(f, "search({}, {})", arg1, arg2),
            TestFunction::Match(arg1, arg2) => write!(f, "match({}, {})", arg1, arg2),
        }
    }
}

/// Enum representing different types of function arguments in a JSONPath query.
#[derive(Debug, Clone)]
enum FnArg {
    /// Represents a literal argument.
    Literal(Literal),
    /// Represents a test argument.
    Test(Box<Test>),
    /// Represents a filter argument.
    Filter(Filter),
}

impl Display for FnArg {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FnArg::Literal(literal) => write!(f, "{}", literal),
            FnArg::Test(test) => write!(f, "{}", test),
            FnArg::Filter(filter) => write!(f, "{}", filter),
        }
    }
}

/// Enum representing different types of literal values in a JSONPath query.
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    /// Represents an integer literal.
    Int(i64),
    /// Represents a floating-point literal.
    Float(f64),
    /// Represents a string literal.
    String(String),
    /// Represents a boolean literal.
    Bool(bool),
    /// Represents a null literal.
    Null,
}

impl Display for Literal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Int(val) => write!(f, "{}", val),
            Literal::Float(val) => write!(f, "{}", val),
            Literal::String(val) => write!(f, "\"{}\"", val),
            Literal::Bool(val) => write!(f, "{}", val),
            Literal::Null => write!(f, "null"),
        }
    }
}