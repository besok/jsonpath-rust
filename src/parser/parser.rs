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

fn parse(rule: Pair<Rule>) -> JsonPath {
    println!(">> {}", rule.to_string());

    match rule.as_rule() {
        Rule::path  => rule.into_inner().next().map(parse).unwrap_or(JsonPath::Empty),
        Rule::chain => JsonPath::Chain(rule.into_inner().map(parse).collect()),
        Rule::root => JsonPath::Root,
        Rule::wildcard => JsonPath::Wildcard,
        Rule::descent => JsonPath::Descent(String::from(rule.into_inner().next().unwrap().as_str())),
        _ => JsonPath::Empty
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dummy_test() {
        let exp = "$.abc.['abc']..abc.[*][*].*..['ac']";
        let path = parse_json_path(exp);
        match path {
            Ok(JsonPath::Chain(_)) => {}
            Err(e) => println!("{}", e.to_string()),
            e => panic!(e)
        }
    }

    #[test]
    fn descent_test() {
        let exp = "..abc";
        let path = parse_json_path(exp);
        match path {
            Ok(JsonPath::Chain(elems)) => {
                assert_eq!(elems, vec![JsonPath::Descent(String::from("abc"))])
            }
            Err(e) => panic!("{:?}", e.to_string()),
            e => panic!(">>> {:?}",e)
        }
    }
}