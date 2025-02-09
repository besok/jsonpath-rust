#![allow(clippy::empty_docs)]

use pest::iterators::{Pair, Pairs};
use crate::parser::errors2::JsonPathParserError;
use crate::parser::model::JsonPath;
use crate::path::JsonLike;
use pest::Parser;
use crate::parser::model2::{Literal, SingularQuery, SingularQuerySegment};

#[derive(Parser)]
#[grammar = "parser/grammar/json_path_9535.pest"]
pub(super) struct JSPathParser;
const MAX_VAL: i64 = 9007199254740991; // Maximum safe integer value in JavaScript
const MIN_VAL: i64 = -9007199254740991; // Minimum safe integer value in JavaScript

pub(super) type Parsed<T> = Result<T, JsonPathParserError>;

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


pub fn singular_query_segments(rule:Pair<Rule>) -> Parsed<Vec<SingularQuerySegment>> {
    let mut segments = vec![];
    for r in rule.into_inner(){
        match r.as_rule(){
            Rule::name_segment => {
                segments.push(SingularQuerySegment::Name(child(r)?.as_str().to_string()));
            }
            Rule::index_segment => {
                segments.push(
                    SingularQuerySegment::Index(child(r)?
                        .as_str()
                        .parse::<i64>()
                        .map_err(|e|(e,"int"))?));
            }
            _ => return Err(r.into())
        }
    }
    Ok(segments)
}

pub fn singular_query(rule:Pair<Rule>) -> Parsed<SingularQuery>{
    let query = child(rule)?;
    let segments = singular_query_segments(child(query.clone())?)?;
    match query.as_rule() {
        Rule::rel_singular_query => Ok(SingularQuery::Current(segments)),
        Rule::abs_singular_query => Ok(SingularQuery::Root(segments)),
        _ => Err(query.into())
    }
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
    let rule_as_str = rule.as_str().to_string();
    rule.into_inner().next().ok_or(JsonPathParserError::EmptyInner(rule_as_str))
}
fn children(rule:Pair<Rule>) -> Pairs<Rule> {
    rule.into_inner()
}
