use std::num::{ParseFloatError, ParseIntError};
use std::str::ParseBoolError;
use pest::iterators::Pair;
use thiserror::Error;
use crate::parser::parser2::Rule;

#[derive(Error, Debug)]
pub enum JsonPathParserError {
    #[error("Failed to parse rule: {0}")]
    PestError(#[from] Box<pest::error::Error<Rule>>),
    #[error("Unexpected rule `{0:?}` when trying to parse logic atom: `{1}` within `{2}`")]
    UnexpectedRuleLogicError(Rule, String, String),
    #[error("Unexpected `none` when trying to parse logic atom: {0} within {1}")]
    UnexpectedNoneLogicError(String, String),
    #[error("Pest returned successful parsing but did not produce any output, that should be unreachable due to .pest definition file: SOI ~ chain ~ EOI")]
    UnexpectedPestOutput,
    #[error("expected a `Rule::path` but found nothing")]
    NoRulePath,
    #[error("expected a `JsonPath::Descent` but found nothing")]
    NoJsonPathDescent,
    #[error("expected a `JsonPath::Field` but found nothing")]
    NoJsonPathField,
    #[error("expected a `f64` or `i64`, but got {0}")]
    InvalidNumber(String),
    #[error("Invalid toplevel rule for JsonPath: {0:?}")]
    InvalidTopLevelRule(Rule),
    #[error("Failed to get inner pairs for {0}")]
    EmptyInner(String),
    #[error("Invalid json path: {0}")]
    InvalidJsonPath(String),
}


impl From<(ParseIntError, &str)> for JsonPathParserError {
    fn from((err, val): (ParseIntError, &str)) -> Self {
        JsonPathParserError::InvalidNumber(format!("{:?} for `{}`", err, val))
    }
}
impl From<(ParseFloatError, &str)> for JsonPathParserError {
    fn from((err, val): (ParseFloatError, &str)) -> Self {
        JsonPathParserError::InvalidNumber(format!("{:?} for `{}`", err, val))
    }
}impl From<ParseBoolError> for JsonPathParserError {
    fn from(err : ParseBoolError) -> Self {
        JsonPathParserError::InvalidJsonPath(format!("{:?} ", err))
    }
}
impl From<Pair<'_, Rule>> for JsonPathParserError {
    fn from(rule: Pair<Rule>) -> Self {
        JsonPathParserError::UnexpectedRuleLogicError(
            rule.as_rule(),
            rule.as_span().as_str().to_string(),
            rule.as_str().to_string())
    }
}
