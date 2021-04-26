use pest::iterators::{Pair, Pairs};
use pest::{Parser, Position};
use serde_json::Value;
use serde_json::json;
use crate::parser::model::{JsonPath, JsonPathIndex, Operand, FilterSign};
use pest::error::{Error, ErrorVariant};

#[derive(Parser)]
#[grammar = "parser/grammar/json_path.pest"]
struct JsonPathParser;

pub fn parse_json_path(jp_str: &str) -> Result<JsonPath, Error<Rule>> {
    Ok(parse_impl(JsonPathParser::parse(Rule::path, jp_str)?.next().unwrap()))
}

fn parse_impl(rule: Pair<Rule>) -> JsonPath {
    println!(">> path {}", rule.to_string());

    match rule.as_rule() {
        Rule::path => rule.into_inner().next().map(parse_impl).unwrap_or(JsonPath::Empty),
        Rule::current => JsonPath::Current(Box::new(rule.into_inner().next().map(parse_impl).unwrap_or(JsonPath::Empty))),
        Rule::chain => JsonPath::Chain(rule.into_inner().map(parse_impl).collect()),
        Rule::root => JsonPath::Root,
        Rule::wildcard => JsonPath::Wildcard,
        Rule::descent => parse_key(down(rule)).map(JsonPath::Descent).unwrap_or(JsonPath::Empty),
        Rule::field => parse_key(down(rule)).map(JsonPath::Field).unwrap_or(JsonPath::Empty),
        Rule::index => JsonPath::Index(parse_index(rule)),
        _ => JsonPath::Empty
    }
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

fn parse_slice(mut pairs: Pairs<Rule>) -> JsonPathIndex {
    let mut start = 0;
    let mut end = 0;
    let mut step = 1;
    while pairs.peek().is_some() {
        let in_pair = pairs.next().unwrap();
        match in_pair.as_rule() {
            Rule::start_slice => start = in_pair.as_str().parse::<i32>().unwrap_or(start),
            Rule::end_slice => end = in_pair.as_str().parse::<i32>().unwrap_or(end),
            Rule::step_slice => step = in_pair.into_inner().next().unwrap().as_str().parse::<usize>().unwrap_or(step),
            _ => ()
        }
    }
    JsonPathIndex::Slice(start, end, step)
}

fn parse_unit_keys(mut pairs: Pairs<Rule>) -> JsonPathIndex {
    let mut keys = vec![];

    while pairs.peek().is_some() {
        keys.push(String::from(pairs.next().unwrap().into_inner().next().unwrap().as_str()));
    }
    JsonPathIndex::UnionKeys(keys)
}

fn parse_unit_indexes(mut pairs: Pairs<Rule>) -> JsonPathIndex {
    let mut keys = vec![];

    while pairs.peek().is_some() {
        keys.push(pairs.next().unwrap().as_str().parse::<f64>().unwrap_or(-1.0));
    }
    JsonPathIndex::UnionIndex(keys)
}

fn parse_filter_index(mut pairs: Pairs<Rule>) -> JsonPathIndex {
    fn process_op(rule: Pair<Rule>) -> Operand {
        println!(">> filter {}", rule.to_string());
        match rule.as_rule() {
            Rule::number => Operand::Static(Value::from(rule.as_str().parse::<f64>().unwrap())),
            Rule::string_qt => Operand::Static(Value::from(rule.into_inner().next().unwrap().as_str())),
            Rule::chain => {
                match parse_impl(rule) {
                   JsonPath::Chain(elems) => {
                        if elems.len() == 1 {
                            match elems.first() {
                                Some(JsonPath::Index(JsonPathIndex::UnionKeys(keys))) => Operand::Static(Value::from(keys.clone())),
                                Some(JsonPath::Index(JsonPathIndex::UnionIndex(keys))) => Operand::Static(Value::from(keys.clone())),
                                _ => Operand::Dynamic(Box::new(JsonPath::Chain(elems)))
                            }
                        } else {
                            Operand::Dynamic(Box::new(JsonPath::Chain(elems)))
                        }
                    }
                    jp => Operand::Dynamic(Box::new(jp))
                }
            }
            _ => Operand::Static(Value::Null)
        }
    }
    let mut left: Operand = process_op(pairs.next().unwrap());
    if pairs.peek().is_none() {
        JsonPathIndex::exists(left)
    } else {
        let sign: FilterSign = FilterSign::new(pairs.next().unwrap().as_str());
        let right: Operand = process_op(pairs.next().unwrap());
        JsonPathIndex::Filter(left, sign, right)
    }
}

fn parse_index(rule: Pair<Rule>) -> JsonPathIndex {
    println!(">> index {}", rule.to_string());
    let next = down(rule);
    match next.as_rule() {
        Rule::unsigned => JsonPathIndex::Single(next.as_str().parse::<usize>().unwrap()),
        Rule::slice => parse_slice(next.into_inner()),
        Rule::unit_indexes => parse_unit_indexes(next.into_inner()),
        Rule::unit_keys => parse_unit_keys(next.into_inner()),
        Rule::filter => parse_filter_index(next.into_inner()),
        _ => JsonPathIndex::Single(next.as_str().parse::<usize>().unwrap())
    }
}

fn down(rule: Pair<Rule>) -> Pair<Rule> {
    rule.into_inner().next().unwrap()
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::panic;
    use crate::parser::model::JsonPath::{Chain, Current, Field, Descent, Wildcard};

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
        test(".abc.*", vec![JsonPath::field("abc"), JsonPath::Wildcard]);
        test(".abc.[*]", vec![JsonPath::field("abc"), JsonPath::Wildcard]);
        test(".abc[*]", vec![JsonPath::field("abc"), JsonPath::Wildcard]);
        test_failed("..*");
        test_failed("abc*");
    }

    #[test]
    fn index__single_test() {
        test("[1]", vec![JsonPath::Index(JsonPathIndex::Single(1))]);
        test_failed("[-1]");
        test_failed("[1a]");
    }

    #[test]
    fn index__slice_test() {
        test("[1:1000:10]", vec![JsonPath::Index(JsonPathIndex::Slice(1, 1000, 10))]);
        test("[:1000:10]", vec![JsonPath::Index(JsonPathIndex::Slice(0, 1000, 10))]);
        test("[:1000]", vec![JsonPath::Index(JsonPathIndex::Slice(0, 1000, 1))]);
        test("[:]", vec![JsonPath::Index(JsonPathIndex::Slice(0, 0, 1))]);
        test("[::10]", vec![JsonPath::Index(JsonPathIndex::Slice(0, 0, 10))]);
        test_failed("[::-1]");
        test_failed("[:::0]");
    }

    #[test]
    fn index_union_test() {
        test("[1,2,3]", vec![JsonPath::Index(JsonPathIndex::UnionIndex(vec![1.0, 2.0, 3.0]))]);
        test("['abc','bcd']", vec![JsonPath::Index(JsonPathIndex::UnionKeys(vec![String::from("abc"), String::from("bcd")]))]);
        test_failed("[]");
        test("[-1,-2]", vec![JsonPath::Index(JsonPathIndex::UnionIndex(vec![-1.0, -1.0]))]);
        test_failed("[abc,bcd]");
        test_failed("[\"abc\",\"bcd\"]");
    }

    #[test]
    fn index_filter_test() {
        test("[?('abc' == 'abc')]", vec![JsonPath::Index(JsonPathIndex::Filter(
            Operand::str("abc"),
            FilterSign::Equal,
            Operand::str("abc"),
        ))]);
        test("[?('abc' == 1)]", vec![JsonPath::Index(JsonPathIndex::Filter(
            Operand::str("abc"),
            FilterSign::Equal,
            Operand::val(json!(1.0)),
        ))]);

        test("[?(@.abc in ['abc','bcd'])]", vec![JsonPath::Index(JsonPathIndex::Filter(
            Operand::Dynamic(Box::new(Chain(vec![Current(Box::new(Chain(vec![Field(String::from("abc"))])))]))),
            FilterSign::In,
            Operand::val(json!(["abc","bcd"])),
        ))]);
        test("[?(@.abc.[*] in ['abc','bcd'])]", vec![JsonPath::Index(JsonPathIndex::Filter(
            Operand::Dynamic(Box::new(Chain(vec![Current(Box::new(Chain(vec![Field(String::from("abc")),Wildcard])))]))),
            FilterSign::In,
            Operand::val(json!(["abc","bcd"])),
        ))]);
        test("[?(@.[*]..next in ['abc','bcd'])]", vec![JsonPath::Index(JsonPathIndex::Filter(
            Operand::Dynamic(Box::new(Chain(vec![Current(Box::new(Chain(vec![Wildcard,Descent(String::from("next"))])))]))),
            FilterSign::In,
            Operand::val(json!(["abc","bcd"])),
        ))]);

        test("[?(@[1] in ['abc','bcd'])]", vec![JsonPath::Index(JsonPathIndex::Filter(
            Operand::Dynamic(Box::new(Chain(vec![Current(Box::new(Chain(vec![JsonPath::Index(JsonPathIndex::Single(1))])))]))),
            FilterSign::In,
            Operand::val(json!(["abc","bcd"])),
        ))]);
    }
}