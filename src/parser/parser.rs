use pest::iterators::{Pair, Pairs};
use pest::{Parser};
use serde_json::{Value};
use crate::parser::model::{JsonPath, JsonPathIndex, Operand, FilterSign};
use pest::error::{Error};

#[derive(Parser)]
#[grammar = "parser/grammar/json_path.pest"]
struct JsonPathParser;

/// the parsing function.
/// Since the parsing can finish with error the result is [[Result]]
pub fn parse_json_path(jp_str: &str) -> Result<JsonPath, Error<Rule>> {
    Ok(parse_internal(JsonPathParser::parse(Rule::path, jp_str)?.next().unwrap()))
}

/// Internal function takes care of the logic by parsing the operators and unrolling the string into the final result.
fn parse_internal(rule: Pair<Rule>) -> JsonPath {
    match rule.as_rule() {
        Rule::path => rule.into_inner().next().map(parse_internal).unwrap_or(JsonPath::Empty),
        Rule::current => JsonPath::Current(Box::new(rule.into_inner().next().map(parse_internal).unwrap_or(JsonPath::Empty))),
        Rule::chain => JsonPath::Chain(rule.into_inner().map(parse_internal).collect()),
        Rule::root => JsonPath::Root,
        Rule::wildcard => JsonPath::Wildcard,
        Rule::descent => parse_key(down(rule)).map(JsonPath::Descent).unwrap_or(JsonPath::Empty),
        Rule::field => parse_key(down(rule)).map(JsonPath::Field).unwrap_or(JsonPath::Empty),
        Rule::index => JsonPath::Index(parse_index(rule)),
        _ => JsonPath::Empty
    }
}

/// parsing the rule 'key' with the structures either .key or .]'key'[
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
            Rule::step_slice => step = down(in_pair).as_str().parse::<usize>().unwrap_or(step),
            _ => ()
        }
    }
    JsonPathIndex::Slice(start, end, step)
}

fn parse_unit_keys(mut pairs: Pairs<Rule>) -> JsonPathIndex {
    let mut keys = vec![];

    while pairs.peek().is_some() {
        keys.push(String::from(down(pairs.next().unwrap()).as_str()));
    }
    JsonPathIndex::UnionKeys(keys)
}

fn number_to_value(number: &str) -> Value {
    number.parse::<i64>().ok().map(Value::from)
        .or_else(|| number.parse::<f64>().ok().map(Value::from))
        .unwrap()
}

fn parse_unit_indexes(mut pairs: Pairs<Rule>) -> JsonPathIndex {
    let mut keys = vec![];

    while pairs.peek().is_some() {
        keys.push(number_to_value(pairs.next().unwrap().as_str()));
    }
    JsonPathIndex::UnionIndex(keys)
}

fn parse_chain_in_operand(rule: Pair<Rule>) -> Operand {
    match parse_internal(rule) {
        JsonPath::Chain(elems) => {
            if elems.len() == 1 {
                match elems.first() {
                    Some(JsonPath::Index(JsonPathIndex::UnionKeys(keys))) => Operand::val(Value::from(keys.clone())),
                    Some(JsonPath::Index(JsonPathIndex::UnionIndex(keys))) => Operand::val(Value::from(keys.clone())),
                    Some(JsonPath::Field(f)) => Operand::val(Value::Array(vec![Value::from(f.clone())])),
                    _ => Operand::Dynamic(Box::new(JsonPath::Chain(elems)))
                }
            } else {
                Operand::Dynamic(Box::new(JsonPath::Chain(elems)))
            }
        }
        jp => Operand::Dynamic(Box::new(jp))
    }
}

fn parse_operand(rule: Pair<Rule>) -> Operand {
    match rule.as_rule() {
        Rule::number => Operand::Static(number_to_value(rule.as_str())),
        Rule::string_qt => Operand::Static(Value::from(down(rule).as_str())),
        Rule::chain => parse_chain_in_operand(rule),
        _ => Operand::Static(Value::Null)
    }
}

fn parse_filter_index(mut pairs: Pairs<Rule>) -> JsonPathIndex {
    let left: Operand = parse_operand(pairs.next().unwrap());
    if pairs.peek().is_none() {
        JsonPathIndex::exists(left)
    } else {
        let sign: FilterSign = FilterSign::new(pairs.next().unwrap().as_str());
        let right: Operand = parse_operand(pairs.next().unwrap());
        JsonPathIndex::Filter(left, sign, right)
    }
}

fn parse_index(rule: Pair<Rule>) -> JsonPathIndex {
    let next = down(rule);
    match next.as_rule() {
        Rule::unsigned => JsonPathIndex::Single(number_to_value(next.as_str())),
        Rule::slice => parse_slice(next.into_inner()),
        Rule::unit_indexes => parse_unit_indexes(next.into_inner()),
        Rule::unit_keys => parse_unit_keys(next.into_inner()),
        Rule::filter => parse_filter_index(next.into_inner()),
        _ => JsonPathIndex::Single(number_to_value(next.as_str()))
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
    use serde_json::{json};

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
        test("$.k.['k']['k']..k..['k'].*.[*][*][1][1,2]['k','k'][:][10:][:10][10:10:10][?(@)][?(@.abc >= 10)]",
             vec![
                 JsonPath::Root,
                 JsonPath::field("k"),
                 JsonPath::field("k"),
                 JsonPath::field("k"),
                 JsonPath::descent("k"),
                 JsonPath::descent("k"),
                 JsonPath::Wildcard,
                 JsonPath::Wildcard,
                 JsonPath::Wildcard,
                 JsonPath::Index(JsonPathIndex::Single(json!(1))),
                 JsonPath::Index(JsonPathIndex::UnionIndex(vec![json!(1), json!(2)])),
                 JsonPath::Index(JsonPathIndex::UnionKeys(vec!["k".to_string(), "k".to_string()])),
                 JsonPath::Index(JsonPathIndex::Slice(0, 0, 1)),
                 JsonPath::Index(JsonPathIndex::Slice(10, 0, 1)),
                 JsonPath::Index(JsonPathIndex::Slice(0, 10, 1)),
                 JsonPath::Index(JsonPathIndex::Slice(10, 10, 10)),
                 JsonPath::Index(JsonPathIndex::Filter(Operand::path(Chain(vec![JsonPath::Current(Box::new(JsonPath::Empty))])), FilterSign::Exists, Operand::path(JsonPath::Empty))),
                 JsonPath::Index(JsonPathIndex::Filter(Operand::path(Chain(vec![JsonPath::Current(Box::new(Chain(vec![JsonPath::field("abc")])))])), FilterSign::GrOrEq, Operand::val(json!(10)))),
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
    fn index_single_test() {
        test("[1]", vec![JsonPath::Index(JsonPathIndex::Single(json!(1)))]);
        test_failed("[-1]");
        test_failed("[1a]");
    }

    #[test]
    fn index_slice_test() {
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
        test("[1,2,3]", vec![JsonPath::Index(JsonPathIndex::UnionIndex(vec![json!(1), json!(2), json!(3)]))]);
        test("['abc','bcd']", vec![JsonPath::Index(JsonPathIndex::UnionKeys(vec![String::from("abc"), String::from("bcd")]))]);
        test_failed("[]");
        test("[-1,-2]", vec![JsonPath::Index(JsonPathIndex::UnionIndex(vec![json!(-1), json!(-2)]))]);
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
            Operand::val(json!(1)),
        ))]);

        test("[?(@.abc in ['abc','bcd'])]", vec![JsonPath::Index(JsonPathIndex::Filter(
            Operand::Dynamic(Box::new(Chain(vec![Current(Box::new(Chain(vec![Field(String::from("abc"))])))]))),
            FilterSign::In,
            Operand::val(json!(["abc","bcd"])),
        ))]);
        test("[?(@.abc.[*] in ['abc','bcd'])]", vec![JsonPath::Index(JsonPathIndex::Filter(
            Operand::Dynamic(Box::new(Chain(vec![Current(Box::new(Chain(vec![Field(String::from("abc")), Wildcard])))]))),
            FilterSign::In,
            Operand::val(json!(["abc","bcd"])),
        ))]);
        test("[?(@.[*]..next in ['abc','bcd'])]", vec![JsonPath::Index(JsonPathIndex::Filter(
            Operand::Dynamic(Box::new(Chain(vec![Current(Box::new(Chain(vec![Wildcard, Descent(String::from("next"))])))]))),
            FilterSign::In,
            Operand::val(json!(["abc","bcd"])),
        ))]);

        test("[?(@[1] in ['abc','bcd'])]", vec![JsonPath::Index(JsonPathIndex::Filter(
            Operand::Dynamic(Box::new(Chain(vec![Current(Box::new(Chain(vec![JsonPath::Index(JsonPathIndex::Single(json!(1)))])))]))),
            FilterSign::In,
            Operand::val(json!(["abc","bcd"])),
        ))]);
        test("[?(@ == 'abc')]", vec![JsonPath::Index(JsonPathIndex::Filter(
            Operand::Dynamic(Box::new(Chain(vec![Current(Box::new(JsonPath::Empty))]))),
            FilterSign::Equal,
            Operand::val(json!("abc")),
        ))]);
        test("[?(@ subsetOf ['abc'])]", vec![JsonPath::Index(JsonPathIndex::Filter(
            Operand::Dynamic(Box::new(Chain(vec![Current(Box::new(JsonPath::Empty))]))),
            FilterSign::SubSetOf,
            Operand::val(json!(["abc"])),
        ))]);
        test("[?(@[1] subsetOf ['abc','abc'])]", vec![JsonPath::Index(JsonPathIndex::Filter(
            Operand::Dynamic(Box::new(Chain(vec![Current(Box::new(JsonPath::Chain(vec![JsonPath::Index(JsonPathIndex::Single(json!(1)))])))]))),
            FilterSign::SubSetOf,
            Operand::val(json!(["abc","abc"])),
        ))]);
        test("[?(@ subsetOf [1,2,3])]", vec![JsonPath::Index(JsonPathIndex::Filter(
            Operand::Dynamic(Box::new(Chain(vec![Current(Box::new(JsonPath::Empty))]))),
            FilterSign::SubSetOf,
            Operand::val(json!([1,2,3])),
        ))]);

        test_failed("[?(@[1] subsetof ['abc','abc'])]");
        test_failed("[?(@ >< ['abc','abc'])]");
        test_failed("[?(@ in {\"abc\":1})]");
    }
}