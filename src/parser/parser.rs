use pest::iterators::{Pair};
use pest::{Parser, Position};

use crate::parser::model::JsonPath;
use pest::error::{Error, ErrorVariant};

#[derive(Parser)]
#[grammar = "parser/grammar/json_path.pest"]
struct JsonPathParser;

pub fn parse_json_path(jp_str: &str) -> Result<JsonPath, Error<Rule>> {
    parse(JsonPathParser::parse(Rule::path, jp_str)?.next().unwrap())
}

fn parse(rule: Pair<Rule>) -> Result<JsonPath, Error<Rule>> {
    println!(">> {}", rule.as_str());

    match rule.as_rule() {
        Rule::path => {
            let x = rule.into_inner().map(parse).collect::<Vec<Result<JsonPath, Error<Rule>>>>();
            JsonPath::Root
        }
        Rule::chain => {
            let x = rule.into_inner().map(parse).collect::<Vec<Result<JsonPath, Error<Rule>>>>();
            JsonPath::Root
        }
        Rule::root => JsonPath::Root,
        Rule::wildcard => JsonPath::Wildcard,
        _ => JsonPath::Root
    };

    Ok(JsonPath::Root)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dummy_test() {
        let exp = "$.abc.['abc']..abc.[*][*].*..['ac']";
        let path = parse_json_path(exp);
        match path {
            Ok(JsonPath::Root) => {}
            Err(e) => panic!(e.line_col),
            e => panic!(e)
        }
    }
}