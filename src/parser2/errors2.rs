use std::num::{ParseFloatError, ParseIntError};
use std::str::ParseBoolError;
use pest::iterators::Pair;
use thiserror::Error;
use crate::parser2::Rule;

#[derive(Error, Debug, PartialEq)]
pub enum JsonPathError {
    #[error("Failed to parse rule: {0}")]
    PestError(#[from] Box<pest::error::Error<Rule>>),
    #[error("Unexpected rule `{0:?}` when trying to parse `{1}`")]
    UnexpectedRuleLogicError(Rule, String),
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

impl JsonPathError {
    pub fn empty(v:&str) -> Self {
        JsonPathError::EmptyInner(v.to_string())
    }
}

impl From<&str> for JsonPathError {
    fn from(val: &str) -> Self {
        JsonPathError::EmptyInner(val.to_string())
    }
}

impl From<(ParseIntError, &str)> for JsonPathError {
    fn from((err, val): (ParseIntError, &str)) -> Self {
        JsonPathError::InvalidNumber(format!("{:?} for `{}`", err, val))
    }
}
impl From<(ParseFloatError, &str)> for JsonPathError {
    fn from((err, val): (ParseFloatError, &str)) -> Self {
        JsonPathError::InvalidNumber(format!("{:?} for `{}`", err, val))
    }
}impl From<ParseBoolError> for JsonPathError {
    fn from(err : ParseBoolError) -> Self {
        JsonPathError::InvalidJsonPath(format!("{:?} ", err))
    }
}
impl From<Pair<'_, Rule>> for JsonPathError {
    fn from(rule: Pair<Rule>) -> Self {
        JsonPathError::UnexpectedRuleLogicError(
            rule.as_rule(),
            rule.as_str().to_string(),

        )
    }
}

