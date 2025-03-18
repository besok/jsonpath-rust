use crate::parser::model::slice_from;
use crate::parser::model::Comparison;
use crate::parser::model::FilterAtom;
use crate::parser::model::FnArg;
use crate::parser::model::JpQuery;
use crate::parser::model::Literal;
use crate::parser::model::Segment;
use crate::parser::model::Selector;
use crate::parser::model::SingularQuery;
use crate::parser::model::SingularQuerySegment;
use crate::parser::model::TestFunction;
use crate::parser::model::{Comparable, Filter};
use crate::parser::Test;
use crate::parser::{
    comp_expr, comparable, filter_atom, function_expr, jp_query, literal, parse_json_path, segment,
    selector, singular_query, singular_query_segments, slice_selector, test, JSPathParser, Parsed,
    Rule,
};
use crate::{
    arg, atom, cmp, comparable, jq, lit, or, q_segment, q_segments, segment, selector,
    singular_query, slice, test, test_fn,
};
use pest::error::Error;
use pest::iterators::Pair;
use pest::Parser;
use std::fmt::Debug;
use std::{panic, vec};

struct TestPair<T> {
    rule: Rule,
    parse_fn: fn(Pair<Rule>) -> Parsed<T>,
}

impl<T: PartialEq + Debug> TestPair<T> {
    fn new(rule: Rule, parse_fn: fn(Pair<Rule>) -> Parsed<T>) -> Self {
        Self { rule, parse_fn }
    }
    fn assert(self, input: &str, expected: T) -> Self {
        match parse(input, self.rule) {
            Ok(e) => {
                assert((self.parse_fn)(e), expected);
            }
            Err(e) => {
                panic!("parsing error `{}`", e);
            }
        }
        self
    }
    fn assert_fail(self, input: &str) -> Self {
        match parse(input, self.rule) {
            Ok(e) => {
                if let Ok(r) = (self.parse_fn)(e) {
                    panic!("expected error, got {:?}", r);
                }
            }
            Err(e) => {
                println!("parsing error `{}`", e);
            }
        }
        self
    }
}

fn parse(input: &str, rule: Rule) -> Result<Pair<Rule>, Error<Rule>> {
    match JSPathParser::parse(rule, input) {
        Ok(e) => Ok(e.into_iter().next().expect("no pairs found")),
        Err(e) => Err(e),
    }
}

fn assert<T>(result: Parsed<T>, expected: T)
where
    T: PartialEq + Debug,
{
    match result {
        Ok(e) => assert_eq!(e, expected),
        Err(e) => {
            panic!("parsing error `{}`", e);
        }
    }
}

#[test]
fn singular_query_segment_test() {
    TestPair::new(Rule::singular_query_segments, singular_query_segments)
        .assert("[\"b\"][\"b\"]", q_segments!([b][b]))
        .assert("[2][1]", q_segments!([2][1]))
        .assert("[2][\"a\"]", q_segments!([2][a]))
        .assert(".a.b", q_segments!(a b))
        .assert(".a.b[\"c\"][1]", q_segments!(a b [c][1]));
}
#[test]
fn singular_query_test() {
    TestPair::new(Rule::singular_query, singular_query)
        .assert("@.a.b", singular_query!(@ a b))
        .assert("@", SingularQuery::Current(vec![]))
        .assert("$", SingularQuery::Root(vec![]))
        .assert("$.a.b.c", singular_query!(a b c))
        .assert("$[\"a\"].b[3]", singular_query!([a] b [3]));
}

#[test]
fn slice_selector_test() {
    TestPair::new(Rule::slice_selector, slice_selector)
        .assert(":", slice!())
        .assert("::", slice!())
        .assert("1:", slice!(1))
        .assert("1:1", slice!(1, 1))
        .assert("1:1:1", slice!(1, 1, 1))
        .assert(":1:1", slice!(,1,1))
        .assert("::1", slice!(,,1))
        .assert("1::1", slice!(1,,1))
        .assert_fail("-0:")
        .assert_fail("9007199254740995");
}

#[test]
fn function_expr_test() {
    TestPair::new(Rule::function_expr, function_expr)
        .assert("length(1)", test_fn!(length arg!(lit!(i 1))))
        .assert("length(true)", test_fn!(length arg!(lit!(b true))))
        .assert(
            "search(@, \"abc\")",
            test_fn!(search arg!(t test!(@ ) ), arg!(lit!(s "abc"))),
        )
        .assert(
            "count(@.a)",
            test_fn!(count arg!(t test!(@ segment!(selector!(a))))),
        )
        .assert_fail("count\t(@.*)");
}

#[test]
fn jq_test() {
    let atom = Filter::Atom(atom!(
        comparable!(> singular_query!(@ a b)),
        ">",
        comparable!(lit!(i 1))
    ));
    TestPair::new(Rule::jp_query, jp_query).assert(
        "$.a.b[?@.a.b > 1]",
        jq!(
            segment!(selector!(a)),
            segment!(selector!(b)),
            segment!(selector!(?atom))
        ),
    );
}

#[test]
fn comp_expr_test() {
    TestPair::new(Rule::comp_expr, comp_expr).assert(
        "@.a.b.c == 1",
        cmp!(
            comparable!(> singular_query!(@ a b c)),
            "==",
            comparable!(lit!(i 1))
        ),
    );
}

#[test]
fn literal_test() {
    TestPair::new(Rule::literal, literal)
        .assert("'☺'", lit!(s "☺"))
        .assert_fail("\"\n\"")
        .assert("' '", lit!(s " "))
        .assert("'\"'", lit!(s "\""))
        .assert("null", lit!())
        .assert("false", lit!(b false))
        .assert("true", lit!(b true))
        .assert("\"hello\"", lit!(s "hello"))
        .assert("\'hello\'", lit!(s "hello"))
        .assert("\'hel\\'lo\'", lit!(s "hel\\'lo"))
        .assert("\'hel\"lo\'", lit!(s "hel\"lo"))
        .assert("\'hel\\nlo\'", lit!(s "hel\\nlo"))
        .assert("1", lit!(i 1))
        .assert("0", lit!(i 0))
        .assert("-0", lit!(i 0))
        .assert("1.2", lit!(f 1.2))
        .assert("9007199254740990", lit!(i 9007199254740990))
        .assert_fail("hel\\\"lo")
        .assert_fail("9007199254740995");
}

#[test]
fn filter_atom_test() {
    TestPair::new(Rule::atom_expr, filter_atom)
        .assert(
            "1 > 2",
            atom!(comparable!(lit!(i 1)), ">", comparable!(lit!(i 2))),
        )
        .assert(
            "!(@.a ==1 || @.b == 2)",
            atom!(!or!(
                Filter::Atom(atom!(
                    comparable!(> singular_query!(@ a)),
                    "==",
                    comparable!(lit!(i 1))
                )),
                Filter::Atom(atom!(
                    comparable!(> singular_query!(@ b)),
                    "==",
                    comparable!(lit!(i 2))
                ))
            )),
        );
}
#[test]
fn comparable_test() {
    TestPair::new(Rule::comparable, comparable)
        .assert("1", comparable!(lit!(i 1)))
        .assert("\"a\"", comparable!(lit!(s "a")))
        .assert("@.a.b.c", comparable!(> singular_query!(@ a b c)))
        .assert("$.a.b.c", comparable!(> singular_query!(a b c)))
        .assert("$[1]", comparable!(> singular_query!([1])))
        .assert("length(1)", comparable!(f test_fn!(length arg!(lit!(i 1)))));
}

#[test]
fn parse_path() {
    let result = parse_json_path("$");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), JpQuery::new(vec![]));
}

#[test]
fn parse_i64() {
    TestPair::new(Rule::literal, literal).assert("1e2", lit!(f 100.0));
}
#[test]
fn parse_selector() {
    TestPair::new(Rule::selector, selector).assert("1:1", Selector::Slice(Some(1), Some(1), None));
}
#[test]
fn parse_global() {
    let sel_a = segment!(selector!(a));
    TestPair::new(Rule::jp_query, jp_query)
        // .assert("$", JpQuery::new(vec![]))
        // .assert("$.a", JpQuery::new(vec![sel_a.clone()]))
        // .assert("$..a", JpQuery::new(vec![segment!(..sel_a)]))
        // .assert(
        //     "$..*",
        //     JpQuery::new(vec![segment!(..segment!(selector!(*)))]),
        // )
        // .assert(
        //     "$[1 :5:2]",
        //     JpQuery::new(vec![segment!(selector!(slice slice!(1, 5, 2)))]),
        // )
        // .assert(
        //     "$['a']['b']",
        //     JpQuery::new(vec![segment!(Selector::Name("'a'".to_string())), segment!(Selector::Name("'b'".to_string()))]),
        // )
        //
        // .assert(
        //     "$[1, 1:1]",
        //     JpQuery::new(vec![Segment::Selectors(vec![
        //         Selector::Index(1),
        //         Selector::Slice(Some(1), Some(1), None),
        //     ])]),
        // )
        // .assert_fail("$..\ra")
    ;
}
