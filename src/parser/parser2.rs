#![allow(clippy::empty_docs)]

use crate::parser::errors2::JsonPathParserError;
use crate::parser::model2::{Comparable, Comparison, Filter, FilterAtom, FnArg, JpQuery, Literal, Segment, SingularQuery, SingularQuerySegment, Test, TestFunction};
use crate::path::JsonLike;
use pest::iterators::{Pair, Pairs};
use pest::Parser;

#[derive(Parser)]
#[grammar = "parser/grammar/json_path_9535.pest"]
pub(super) struct JSPathParser;
const MAX_VAL: i64 = 9007199254740991; // Maximum safe integer value in JavaScript
const MIN_VAL: i64 = -9007199254740991; // Minimum safe integer value in JavaScript

pub(super) type Parsed<T> = Result<T, JsonPathParserError>;

pub fn jp_query(rule: Pair<Rule>) -> Parsed<JpQuery> {
    Ok(JpQuery::new(segments(rule)?))
}
pub fn rel_query(rule: Pair<Rule>) -> Parsed<Vec<Segment>> {
    segments(rule)
}

pub fn segments(rule: Pair<Rule>) -> Parsed<Vec<Segment>> {
    rule.into_inner().map(segment).collect()
}

pub fn segment(rule: Pair<Rule>) -> Parsed<Segment> {
    unimplemented!()
}

pub fn function_expr(rule: Pair<Rule>) -> Parsed<TestFunction> {
    let mut elems = children(rule);
    let name = elems
        .next()
        .map(|e| e.as_str())
        .ok_or(JsonPathParserError::empty("function expression"))?
        ;
    let mut args = vec![];
    for arg in elems {
        match arg.as_rule() {
            Rule::literal => args.push(FnArg::Literal(literal(arg)?)),
            Rule::test => args.push(FnArg::Test(Box::new(test(arg)?))),
            Rule::logical_expr_or => args.push(FnArg::Filter(logical_expr(arg)?)),

            _ => return Err(arg.into()),
        }
    }

    TestFunction::try_new(name, args)
}

pub fn test(rule: Pair<Rule>) -> Parsed<Test> {
    let child = child(rule)?;
    match child.as_rule() {
        Rule::jp_query => Ok(Test::AbsQuery(jp_query(child)?)),
        Rule::rel_query => Ok(Test::RelQuery(rel_query(child)?)),
        Rule::function_expr => Ok(Test::Function(Box::new(function_expr(child)?))),
        _ => Err(child.into()),
    }
}

pub fn logical_expr(rule: Pair<Rule>) -> Parsed<Filter> {
    unimplemented!()
}

pub fn singular_query_segments(rule: Pair<Rule>) -> Parsed<Vec<SingularQuerySegment>> {
    let mut segments = vec![];
    for r in rule.into_inner() {
        match r.as_rule() {
            Rule::name_segment => {
                segments.push(SingularQuerySegment::Name(child(r)?.as_str().to_string()));
            }
            Rule::index_segment => {
                segments.push(SingularQuerySegment::Index(
                    child(r)?.as_str().parse::<i64>().map_err(|e| (e, "int"))?,
                ));
            }
            _ => return Err(r.into()),
        }
    }
    Ok(segments)
}
fn validate_range(val: i64) -> Result<i64, JsonPathParserError> {
    if val > MAX_VAL || val < MIN_VAL {
        Err(JsonPathParserError::InvalidJsonPath(format!(
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
    let get_int = |r: Pair<Rule>| r.as_str().parse::<i64>().map_err(|e| (e, "int"));

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
    let query = child(rule)?;
    let segments = singular_query_segments(child(query.clone())?)?;
    match query.as_rule() {
        Rule::rel_singular_query => Ok(SingularQuery::Current(segments)),
        Rule::abs_singular_query => Ok(SingularQuery::Root(segments)),
        _ => Err(query.into()),
    }
}

pub fn comp_expr(rule:Pair<Rule>) -> Parsed<Comparison> {
    let mut children = rule.into_inner();

    let lhs = comparable(children.next().ok_or(JsonPathParserError::empty("comparison"))?)?;
    let op = children.next().ok_or(JsonPathParserError::empty("comparison"))?.as_str();
    let rhs = comparable(children.next().ok_or(JsonPathParserError::empty("comparison"))?)?;;

    Comparison::try_new(op,lhs, rhs)
}

pub fn literal(rule: Pair<Rule>) -> Parsed<Literal> {
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

    match first.as_rule() {
        Rule::string => Ok(Literal::String(first.as_str().to_string())),
        Rule::number => parse_number(first.as_str()),
        Rule::bool => Ok(Literal::Bool(first.as_str().parse::<bool>()?)),
        Rule::null => Ok(Literal::Null),

        _ => Err(first.into()),
    }
}

pub fn filter_atom(pair: Pair<Rule>) -> Parsed<FilterAtom> {
    let rule = child(pair)?;
    match rule.as_rule() {
        Rule::paren_expr =>  {
            let mut not = false;
            let mut logic_expr = None;
            for r in rule.into_inner(){
                match r.as_rule(){
                    Rule::not_op => not = true,
                    Rule::logical_expr_or =>  logic_expr = Some(logical_expr(r)?),
                    _ => (),
                }
            }

            logic_expr
                .map(|expr|FilterAtom::filter(expr, not))
                .ok_or("Logical expression is absent".into())
        }
        Rule::comp_expr =>  {
            Ok(FilterAtom::cmp(Box::new(comp_expr(rule)?)))
        }
        Rule::test_expr => {
            let mut not = false;
            let mut test_expr = None;
            for r in rule.into_inner(){
                match r.as_rule(){
                    Rule::not_op => not = true,
                    Rule::test =>  test_expr = Some(test(r)?),
                    _ => (),
                }
            }

            test_expr
                .map(|expr|FilterAtom::test(expr, not))
                .ok_or("Logical expression is absent".into())
        }
        _ => Err(rule.into()),
    }
}

pub fn comparable(rule: Pair<Rule>) -> Parsed<Comparable>{
    let rule = child(rule)?;
    match rule.as_rule(){
        Rule::literal => Ok(Comparable::Literal(literal(rule)?)),
        Rule::singular_query => Ok(Comparable::SingularQuery(singular_query(rule)?)),
        Rule::function_expr => Ok(Comparable::Function(function_expr(rule)?)),
        _ => Err(rule.into())
    }
}

fn child(rule: Pair<Rule>) -> Parsed<Pair<Rule>> {
    let rule_as_str = rule.as_str().to_string();
    rule.into_inner()
        .next()
        .ok_or(JsonPathParserError::EmptyInner(rule_as_str))
}

fn children(rule: Pair<Rule>) -> Pairs<Rule> {
    rule.into_inner()
}
