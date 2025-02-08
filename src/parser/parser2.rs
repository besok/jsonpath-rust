#![allow(clippy::empty_docs)]

use crate::parser::errors::JsonPathParserError;
use crate::parser::model::JsonPath;
use crate::path::JsonLike;
use pest::Parser;

#[derive(Parser)]
#[grammar = "parser/grammar/json_path_9535.pest"]
struct JSPathParser;
const MAX_VAL: i64 = 9007199254740991; // Maximum safe integer value in JavaScript
const MIN_VAL: i64 = -9007199254740991; // Minimum safe integer value in JavaScript
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



#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::panic;

    fn should_fail(input: &str) {
        if let Ok(elem) = parse_json_path::<Value>(input) {
            panic!("should be false but got {:?}", elem);
        }
    }

    fn assert_jspath<T>(input: &str, expected: JsonPath<T>)
    where
        T: JsonLike,
    {
        match parse_json_path::<T>(input) {
            Ok(e) => assert_eq!(e, expected),
            Err(e) => {
                panic!("parsing error {}", e);
            }
        }
    }


    fn assert_rule(rule:Rule, input:&str, expected:&str){
        match JSPathParser::parse(rule, input) {
            Ok(e) => assert_eq!(e.as_str(), expected),
            Err(e) => {
                panic!("parsing error {}", e);
            }
        }
    }
    fn fail_rule(rule:Rule, input:&str){
        match JSPathParser::parse(rule, input) {
            Ok(e) =>  panic!("should be false but got {:?}", e),
            Err(e) => {}
        }
    }


    #[test]
    fn root_test(){
        assert_rule(Rule::root, "$", "");
        fail_rule(Rule::root, "a");
    }
}
