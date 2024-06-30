use thiserror::Error;

use super::parser::Rule;

#[derive(Error, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum JsonPathParserError {
    #[error("Failed to parse rule: {0}")]
    PestError(#[from] pest::error::Error<Rule>),
    #[error("Failed to parse JSON: {0}")]
    JsonParsingError(#[from] serde_json::Error),
    #[error("Unexpected rule {0:?} when trying to parse logic atom: {1} within {2}")]
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
}
