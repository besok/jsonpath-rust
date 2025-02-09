use crate::parser::model2::Literal;
use std::fmt::Debug;
use pest::error::Error;
use pest::iterators::Pair;
use pest::Parser;
use crate::lit;
use crate::parser::parser2::{literal, Rule};
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