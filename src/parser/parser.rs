use pest::iterators::{Pair};
use pest::{Parser, Position};

use crate::parser::model::JsonPath;
use pest::error::{Error, ErrorVariant};

#[derive(Parser)]
#[grammar = "parser/grammar/json_path.pest"]
struct JsonPathParser;

pub fn parse_json_path(jp_str: &str) -> Result<JsonPath, Error<Rule>> {
    Ok(parse(JsonPathParser::parse(Rule::path, jp_str)?.next().unwrap()))
}

fn parse_key(rule: Pair<Rule>) -> Option<String> {
    match rule.as_rule() {
        Rule::key
        | Rule::key_unlim
        | Rule::string_qt => parse_key(down(rule)),
        Rule::key_lim
        | Rule::inner => Some(String::from(rule.as_str())),
        _ => None
    }
}


fn down(rule: Pair<Rule>) -> Pair<Rule> {
    rule.into_inner().next().unwrap()
}

fn parse(rule: Pair<Rule>) -> JsonPath {
    println!(">> {}", rule.to_string());

    match rule.as_rule() {
        Rule::path => rule.into_inner().next().map(parse).unwrap_or(JsonPath::Empty),
        Rule::chain => JsonPath::Chain(rule.into_inner().map(parse).collect()),
        Rule::root => JsonPath::Root,
        Rule::wildcard => JsonPath::Wildcard,
        Rule::descent => parse_key(down(rule)).map(JsonPath::Descent).unwrap_or(JsonPath::Empty),
        Rule::field => parse_key(down(rule)).map(JsonPath::Field).unwrap_or(JsonPath::Empty),
        _ => JsonPath::Empty
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic;

    fn test_failed(input: &str) {
        match parse_json_path(input) {
            Ok(elem) => panic!("should be false but got {:?}", elem),
            Err(e) => println!("{}", e.to_string())
        }
    }

    fn test(input: &str, expected: Vec<JsonPath>) {
        match parse_json_path(input) {
            Ok(JsonPath::Chain(elems)) => assert_eq!(elems, expected),
            Ok(e) => panic!("unexpected value {:?}", e),
            Err(e) => {
                println!("{}", e.to_string());
                panic!("parsing error");
            }
        }
    }

    #[test]
    fn path_test() {
        test("$.abc.['abc']..abc.[*][*].*..['ac']",
             vec![
                 JsonPath::Root,
                 JsonPath::field("abc"),
                 JsonPath::field("abc"),
                 JsonPath::descent("abc"),
                 JsonPath::Wildcard,
                 JsonPath::Wildcard,
                 JsonPath::Wildcard,
                 JsonPath::descent("ac"),

             ])
    }

    #[test]
    fn descent_test() {
        test("..abc", vec![JsonPath::descent("abc")]);
        test("..['abc']", vec![JsonPath::descent("abc")]);
        test_failed("...['abc']");
        test_failed("...abc");
    }

    #[test]
    fn field_test() {
        test(".abc", vec![JsonPath::field("abc")]);
        test(".['abc']", vec![JsonPath::field("abc")]);
        test("['abc']", vec![JsonPath::field("abc")]);
        test(".['abc\\\"abc']", vec![JsonPath::field("abc\\\"abc")]);
        test_failed(".abc()abc");
        test_failed("..[abc]");
        test_failed(".'abc'");
    }

    #[test]
    fn wildcard_test() {
        test(".*", vec![JsonPath::Wildcard]);
        test(".[*]", vec![JsonPath::Wildcard]);
        test(".abc.*", vec![JsonPath::field("abc"),JsonPath::Wildcard]);
        test(".abc.[*]", vec![JsonPath::field("abc"),JsonPath::Wildcard]);
        test(".abc[*]", vec![JsonPath::field("abc"),JsonPath::Wildcard]);
        test_failed("..*");
        test_failed("abc*");
    }
}