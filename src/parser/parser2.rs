#![allow(clippy::empty_docs)]

use pest::iterators::{Pair, Pairs};
use crate::parser::errors2::JsonPathParserError;
use crate::parser::model::JsonPath;
use crate::path::JsonLike;
use pest::Parser;
use crate::parser::model2::Literal;

#[derive(Parser)]
#[grammar = "parser/grammar/json_path_9535.pest"]
struct JSPathParser;
const MAX_VAL: i64 = 9007199254740991; // Maximum safe integer value in JavaScript
const MIN_VAL: i64 = -9007199254740991; // Minimum safe integer value in JavaScript

type Parsed<T> = Result<T, JsonPathParserError>;

/// Parses a string into a [JsonPath].
///
/// # Errors
///
/// Returns a variant of [JsonPathParserError] if the parsing operation failed.
pub fn parse_json_path<T>(jp_str: &str) -> Result<JsonPath<T>, JsonPathParserError>
where
    T: JsonLike,
{
  Ok(JsonPath::Empty)
}



pub fn literal(rule:Pair<Rule>) -> Parsed<Literal> {
    fn parse_number(num: &str) -> Parsed<Literal> {
        if num.contains('.') {
            Ok(Literal::Float(num.parse::<f64>().map_err(|e| (e, num))?))
        } else {
            let num = num.parse::<i64>().map_err(|e| (e, num))?;
            if num > MAX_VAL || num < MIN_VAL {
                Err(JsonPathParserError::InvalidNumber(format!(
                    "number out of bounds: {}",
                    num
                )))
            } else {
                Ok(Literal::Int(num))
            }
        }
    }
    let first = child(rule)?;

    match first.as_rule(){
        Rule::string => Ok(Literal::String(first.as_str().to_string())),
        Rule::number => parse_number(first.as_str()),
        Rule::bool => Ok(Literal::Bool(first.as_str().parse::<bool>()?)),
        Rule::null => Ok(Literal::Null),

        _ => Err(first.into())
    }
}


fn child(rule:Pair<Rule>) -> Parsed<Pair<Rule>> {
    let string = rule.as_str().to_string();
    rule.into_inner().next().ok_or(JsonPathParserError::EmptyInner(string))
}
fn children(rule:Pair<Rule>) -> Pairs<Rule> {
    rule.into_inner()
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use super::*;
    use std::panic;
    use pest::error::Error;
    use crate::lit;

    struct TestPair<T> {
        rule: Rule,
        parse_fn: fn(Pair<Rule>) -> Parsed<T>,
    }

    impl<T:PartialEq + Debug> TestPair<T> {
        fn new(rule: Rule, parse_fn: fn(Pair<Rule>) -> Parsed<T>) -> Self {
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
        match JSPathParser::parse(rule, input){
            Ok(e) => {
                Ok(e.into_iter().next().expect("no pairs found"))
            },
            Err(e) => {
                Err(e)
            }
        }
    }

    fn assert<T>(result: Parsed<T>, expected:T)
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
}
