use crate::parser::model2::SingularQuery;
use crate::parser::model2::SingularQuerySegment;
use crate::parser::model2::Literal;
use std::fmt::Debug;
use pest::error::Error;
use pest::iterators::Pair;
use pest::Parser;
use crate::{lit, q_segments, q_segment, singular_query, slice};
use crate::parser::parser2::{literal, singular_query, singular_query_segments, slice_selector, Rule};
use std::panic;

struct TestPair<T> {
    rule: Rule,
    parse_fn: fn(Pair<Rule>) -> crate::parser::parser2::Parsed<T>,
}

impl<T:PartialEq + Debug> TestPair<T> {
    fn new(rule: Rule, parse_fn: fn(Pair<Rule>) -> crate::parser::parser2::Parsed<T>) -> Self {
        Self {
            rule,
            parse_fn
        }
    }
    fn assert(self,input:&str, expected:T) -> Self {
        match parse(input, self.rule){
            Ok(e) => {
                assert((self.parse_fn)(e), expected);
            },
            Err(e) => {
                panic!("parsing error `{}`", e);
            }
        }
        self
    }
    fn assert_fail(self,input:&str) -> Self {
        match parse(input, self.rule){
            Ok(e) => {
                if let Ok(r) = (self.parse_fn)(e) {
                    panic!("expected error, got {:?}", r);
                }
            },
            Err(_) => {}
        }
        self
    }
}

fn parse(input:&str,rule:Rule) -> Result<Pair<Rule>, Error<Rule>> {
    match crate::parser::parser2::JSPathParser::parse(rule, input){
        Ok(e) => {
            Ok(e.into_iter().next().expect("no pairs found"))
        },
        Err(e) => {
            Err(e)
        }
    }
}

fn assert<T>(result: crate::parser::parser2::Parsed<T>, expected:T)
where T:PartialEq + Debug {
    match result {
        Ok(e) => assert_eq!(e, expected),
        Err(e) => {
            panic!("parsing error `{}`", e);
        }
    }
}


#[test]
fn literals(){

    TestPair::new(Rule::literal, literal)
        .assert("null", lit!())
        .assert("false", lit!(b false))
        .assert("true", lit!(b true))
        .assert("\"hello\"", lit!(s "\"hello\""))
        .assert("\'hello\'", lit!(s "\'hello\'"))
        .assert("\'hel\\'lo\'", lit!(s "\'hel\\'lo\'"))
        .assert("\'hel\"lo\'", lit!(s "\'hel\"lo\'"))
        .assert("\'hel\nlo\'", lit!(s "\'hel\nlo\'"))
        .assert("\'\"\'", lit!(s "\'\"\'"))
        .assert_fail("\'hel\\\"lo\'")
        .assert("1", lit!(i 1))
        .assert("0", lit!(i 0))
        .assert("-0", lit!(i 0))
        .assert("1.2", lit!(f 1.2))
        .assert("9007199254740990", lit!(i 9007199254740990))
        .assert_fail("9007199254740995")
    ;


}

#[test]
fn singular_query_segment_test(){
    TestPair::new(Rule::singular_query_segments, singular_query_segments)
        .assert("[\"b\"][\"b\"]",q_segments!([b][b]))
        .assert("[2][1]",q_segments!([2][1]))
        .assert("[2][\"a\"]",q_segments!([2][a]))
        .assert(".a.b",q_segments!(a b))
        .assert(".a.b[\"c\"][1]",q_segments!(a b [c][1]))
    ;
}
#[test]
fn singular_query_test(){
    TestPair::new(Rule::singular_query, singular_query)
        .assert("@.a.b",singular_query!(@ a b))
        .assert("@",SingularQuery::Current(vec![]))
        .assert("$",SingularQuery::Root(vec![]))
        .assert("$.a.b.c",singular_query!(a b c))
        .assert("$[\"a\"].b[3]",singular_query!([a] b [3]))

    ;
}

#[test]
fn slice_selector_test(){
    TestPair::new(Rule::slice_selector, slice_selector)
        .assert(":",slice!())
        .assert("::",slice!())
        .assert("1:",slice!(1))
        .assert("1:1",slice!(1,1))
        .assert("1:1:1",slice!(1,1,1))
        .assert(":1:1",slice!(,1,1))
        .assert("::1",slice!(,,1))
        .assert("1::1",slice!(1,,1))
        .assert_fail("-0:")
        .assert_fail("9007199254740995")

    ;
}