#![allow(clippy::empty_docs)]
pub mod errors;
mod macros;
pub mod model;
mod tests;

use crate::parser::errors::JsonPathError;
use crate::parser::model::{
    Comparable, Comparison, Filter, FilterAtom, FnArg, JpQuery, Literal, Segment, Selector,
    SingularQuery, SingularQuerySegment, Test, TestFunction,
};

use pest::iterators::Pair;
use pest::Parser;

#[derive(Parser)]
#[grammar = "parser/grammar/json_path_9535.pest"]
pub(super) struct JSPathParser;
const MAX_VAL: i64 = 9007199254740991; // Maximum safe integer value in JavaScript
const MIN_VAL: i64 = -9007199254740991; // Minimum safe integer value in JavaScript

pub(super) type Parsed<T> = Result<T, JsonPathError>;

/// Parses a string into a [JsonPath].
///
/// # Errors
///
/// Returns a variant of [crate::JsonPathParserError] if the parsing operation failed.
pub fn parse_json_path(jp_str: &str) -> Parsed<JpQuery> {
    if jp_str != jp_str.trim() {
        Err(JsonPathError::InvalidJsonPath(
            "Leading or trailing whitespaces".to_string(),
        ))
    } else {
        JSPathParser::parse(Rule::main, jp_str)
            .map_err(Box::new)?
            .next()
            .ok_or(JsonPathError::UnexpectedPestOutput)
            .and_then(next_down)
            .and_then(jp_query)
    }
}

pub fn jp_query(rule: Pair<Rule>) -> Parsed<JpQuery> {
    Ok(JpQuery::new(segments(next_down(rule)?)?))
}
pub fn rel_query(rule: Pair<Rule>) -> Parsed<Vec<Segment>> {
    segments(next_down(rule)?)
}

pub fn segments(rule: Pair<Rule>) -> Parsed<Vec<Segment>> {
    let mut segments = vec![];
    for r in rule.into_inner() {
        segments.push(segment(next_down(r)?)?);
    }
    Ok(segments)
}

pub fn child_segment(rule: Pair<Rule>) -> Parsed<Segment> {
    match rule.as_rule() {
        Rule::wildcard_selector => Ok(Segment::Selector(Selector::Wildcard)),
        Rule::member_name_shorthand => Ok(Segment::name(rule.as_str().trim())),
        Rule::bracketed_selection => {
            let mut selectors = vec![];
            for r in rule.into_inner() {
                selectors.push(selector(r)?);
            }
            if selectors.len() == 1 {
                Ok(Segment::Selector(
                    selectors
                        .into_iter()
                        .next()
                        .ok_or(JsonPathError::empty("selector"))?,
                ))
            } else {
                Ok(Segment::Selectors(selectors))
            }
        }
        _ => Err(rule.into()),
    }
}

pub fn segment(child: Pair<Rule>) -> Parsed<Segment> {
    match child.as_rule() {
        Rule::child_segment => {
            let val = child.as_str().strip_prefix(".").unwrap_or_default();
            if val != val.trim_start() {
                Err(JsonPathError::InvalidJsonPath(format!(
                    "Invalid child segment `{}`",
                    child.as_str()
                )))
            } else {
                child_segment(next_down(child)?)
            }
        }
        Rule::descendant_segment => {
            if child
                .as_str()
                .chars()
                .nth(2)
                .ok_or(JsonPathError::empty(child.as_str()))?
                .is_whitespace()
            {
                Err(JsonPathError::InvalidJsonPath(format!(
                    "Invalid descendant segment `{}`",
                    child.as_str()
                )))
            } else {
                Ok(Segment::Descendant(Box::new(child_segment(next_down(
                    child,
                )?)?)))
            }
        }
        _ => Err(child.into()),
    }
}

pub fn selector(rule: Pair<Rule>) -> Parsed<Selector> {
    let child = next_down(rule)?;
    match child.as_rule() {
        Rule::name_selector => Ok(Selector::Name(child.as_str().trim().to_string())),
        Rule::wildcard_selector => Ok(Selector::Wildcard),
        Rule::index_selector => Ok(Selector::Index(validate_range(
            child
                .as_str()
                .trim()
                .parse::<i64>()
                .map_err(|e| (e, "wrong integer"))?,
        )?)),
        Rule::slice_selector => {
            let (start, end, step) = slice_selector(child)?;
            Ok(Selector::Slice(start, end, step))
        }
        Rule::filter_selector => Ok(Selector::Filter(logical_expr(next_down(child)?)?)),
        _ => Err(child.into()),
    }
}

pub fn function_expr(rule: Pair<Rule>) -> Parsed<TestFunction> {
    let fn_str = rule.as_str();
    let mut elems = rule.into_inner();
    let name = elems
        .next()
        .map(|e| e.as_str())
        .ok_or(JsonPathError::empty("function expression"))?;

    // Check if the function name is valid namely nothing between the name and the opening parenthesis
    if fn_str
        .chars()
        .nth(name.len())
        .map(|c| c != '(')
        .unwrap_or_default()
    {
        Err(JsonPathError::InvalidJsonPath(format!(
            "Invalid function expression `{}`",
            fn_str
        )))
    } else {
        let mut args = vec![];
        for arg in elems {
            let next = next_down(arg)?;
            match next.as_rule() {
                Rule::literal => args.push(FnArg::Literal(literal(next)?)),
                Rule::test => args.push(FnArg::Test(Box::new(test(next)?))),
                Rule::logical_expr => args.push(FnArg::Filter(logical_expr(next)?)),

                _ => return Err(next.into()),
            }
        }

        TestFunction::try_new(name, args)
    }
}

pub fn test(rule: Pair<Rule>) -> Parsed<Test> {
    let child = next_down(rule)?;
    match child.as_rule() {
        Rule::jp_query => Ok(Test::AbsQuery(jp_query(child)?)),
        Rule::rel_query => Ok(Test::RelQuery(rel_query(child)?)),
        Rule::function_expr => Ok(Test::Function(Box::new(function_expr(child)?))),
        _ => Err(child.into()),
    }
}

pub fn logical_expr(rule: Pair<Rule>) -> Parsed<Filter> {
    let mut ors = vec![];
    for r in rule.into_inner() {
        ors.push(logical_expr_and(r)?);
    }
    if ors.len() == 1 {
        Ok(ors
            .into_iter()
            .next()
            .ok_or(JsonPathError::empty("logical expression"))?)
    } else {
        Ok(Filter::Or(ors))
    }
}

pub fn logical_expr_and(rule: Pair<Rule>) -> Parsed<Filter> {
    let mut ands = vec![];
    for r in rule.into_inner() {
        ands.push(Filter::Atom(filter_atom(r)?));
    }
    if ands.len() == 1 {
        Ok(ands
            .into_iter()
            .next()
            .ok_or(JsonPathError::empty("logical expression"))?)
    } else {
        Ok(Filter::And(ands))
    }
}

pub fn singular_query_segments(rule: Pair<Rule>) -> Parsed<Vec<SingularQuerySegment>> {
    let mut segments = vec![];
    for r in rule.into_inner() {
        match r.as_rule() {
            Rule::name_segment => {
                segments.push(SingularQuerySegment::Name(
                    next_down(r)?.as_str().trim().to_string(),
                ));
            }
            Rule::index_segment => {
                segments.push(SingularQuerySegment::Index(
                    next_down(r)?
                        .as_str()
                        .trim()
                        .parse::<i64>()
                        .map_err(|e| (e, "int"))?,
                ));
            }
            _ => return Err(r.into()),
        }
    }
    Ok(segments)
}
fn validate_range(val: i64) -> Result<i64, JsonPathError> {
    if val > MAX_VAL || val < MIN_VAL {
        Err(JsonPathError::InvalidJsonPath(format!(
            "Value {} is out of range",
            val
        )))
    } else {
        Ok(val)
    }
}
pub fn slice_selector(rule: Pair<Rule>) -> Parsed<(Option<i64>, Option<i64>, Option<i64>)> {
    let mut start = None;
    let mut end = None;
    let mut step = None;
    let get_int = |r: Pair<Rule>| r.as_str().trim().parse::<i64>().map_err(|e| (e, "int"));

    for r in rule.into_inner() {
        match r.as_rule() {
            Rule::start => start = Some(validate_range(get_int(r)?)?),
            Rule::end => end = Some(validate_range(get_int(r)?)?),
            Rule::step => {
                step = {
                    if let Some(int) = r.into_inner().next() {
                        Some(validate_range(get_int(int)?)?)
                    } else {
                        None
                    }
                }
            }

            _ => return Err(r.into()),
        }
    }
    Ok((start, end, step))
}

pub fn singular_query(rule: Pair<Rule>) -> Parsed<SingularQuery> {
    let query = next_down(rule)?;
    let segments = singular_query_segments(next_down(query.clone())?)?;
    match query.as_rule() {
        Rule::rel_singular_query => Ok(SingularQuery::Current(segments)),
        Rule::abs_singular_query => Ok(SingularQuery::Root(segments)),
        _ => Err(query.into()),
    }
}

pub fn comp_expr(rule: Pair<Rule>) -> Parsed<Comparison> {
    let mut children = rule.into_inner();

    let lhs = comparable(children.next().ok_or(JsonPathError::empty("comparison"))?)?;
    let op = children
        .next()
        .ok_or(JsonPathError::empty("comparison"))?
        .as_str();
    let rhs = comparable(children.next().ok_or(JsonPathError::empty("comparison"))?)?;

    Comparison::try_new(op, lhs, rhs)
}

pub fn literal(rule: Pair<Rule>) -> Parsed<Literal> {
    fn parse_number(num: &str) -> Parsed<Literal> {
        let num = num.trim();

        if num.contains('.') || num.contains('e') || num.contains('E') {
            Ok(Literal::Float(num.parse::<f64>().map_err(|e| (e, num))?))
        } else {
            let num = num.trim().parse::<i64>().map_err(|e| (e, num))?;
            if num > MAX_VAL || num < MIN_VAL {
                Err(JsonPathError::InvalidNumber(format!(
                    "number out of bounds: {}",
                    num
                )))
            } else {
                Ok(Literal::Int(num))
            }
        }
    }
    let first = next_down(rule)?;

    match first.as_rule() {
        Rule::string => Ok(Literal::String(
            first
                .as_str()
                .trim_matches(|c| c == '\'' || c == '"')
                .trim()
                .to_owned(),
        )),
        Rule::number => parse_number(first.as_str()),
        Rule::bool => Ok(Literal::Bool(first.as_str().parse::<bool>()?)),
        Rule::null => Ok(Literal::Null),

        _ => Err(first.into()),
    }
}

pub fn filter_atom(pair: Pair<Rule>) -> Parsed<FilterAtom> {
    let rule = next_down(pair)?;

    match rule.as_rule() {
        Rule::paren_expr => {
            let mut not = false;
            let mut logic_expr = None;
            for r in rule.into_inner() {
                match r.as_rule() {
                    Rule::not_op => not = true,
                    Rule::logical_expr => logic_expr = Some(logical_expr(r)?),
                    _ => (),
                }
            }

            logic_expr
                .map(|expr| FilterAtom::filter(expr, not))
                .ok_or("Logical expression is absent".into())
        }
        Rule::comp_expr => Ok(FilterAtom::cmp(Box::new(comp_expr(rule)?))),
        Rule::test_expr => {
            let mut not = false;
            let mut test_expr = None;
            for r in rule.into_inner() {
                match r.as_rule() {
                    Rule::not_op => not = true,
                    Rule::test => test_expr = Some(test(r)?),
                    _ => (),
                }
            }

            test_expr
                .map(|expr| FilterAtom::test(expr, not))
                .ok_or("Logical expression is absent".into())
        }
        _ => Err(rule.into()),
    }
}

pub fn comparable(rule: Pair<Rule>) -> Parsed<Comparable> {
    let rule = next_down(rule)?;
    match rule.as_rule() {
        Rule::literal => Ok(Comparable::Literal(literal(rule)?)),
        Rule::singular_query => Ok(Comparable::SingularQuery(singular_query(rule)?)),
        Rule::function_expr => {
            let tf = function_expr(rule)?;
            if tf.is_comparable() {
                Ok(Comparable::Function(tf))
            } else {
                Err(JsonPathError::InvalidJsonPath(format!(
                    "Function {} is not comparable",
                    tf.to_string()
                )))
            }
        }
        _ => Err(rule.into()),
    }
}

fn next_down(rule: Pair<Rule>) -> Parsed<Pair<Rule>> {
    let rule_as_str = rule.as_str().to_string();
    rule.into_inner()
        .next()
        .ok_or(JsonPathError::InvalidJsonPath(rule_as_str))
}
