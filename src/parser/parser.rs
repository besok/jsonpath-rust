use pest::iterators::{Pair};
use pest::{Parser};

use crate::parser::model::JsonPath;
use pest::error::Error;

#[derive(Parser)]
#[grammar = "parser/grammar/json_path.pest"]
struct JsonPathParser;

pub fn parse_json_path(jp_str: &str) -> Result<JsonPath,Error<Rule>> {
    let path = JsonPathParser::parse(Rule::single_path, jp_str)?.next().unwrap();
    let path = parse(path)?;
    Ok(path)
}

fn parse(element: Pair<Rule>) -> Result<JsonPath,Error<Rule>> {
    println!(">> {}",element.as_str());
    Ok(JsonPath::Root)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_root() {
        let exp = "$.abc.abc";
        let path = parse_json_path(exp);
        match  path {
            Ok(JsonPath::Root) => {}
            e => panic!(e)
        }
    }
}