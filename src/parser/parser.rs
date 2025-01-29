#![allow(clippy::empty_docs)]

use crate::parser::errors::JsonPathParserError;
use crate::parser::model::FilterExpression::{And, Not, Or};
use crate::parser::model::{
    FilterExpression, FilterSign, Function, JsonPath, JsonPathIndex, Operand,
};
use crate::path::JsonLike;
use pest::iterators::{Pair, Pairs};
use pest::Parser;

#[derive(Parser)]
#[grammar = "parser/grammar/json_path.pest"]
struct JsonPathParser;

/// Parses a string into a [JsonPath].
///
/// # Errors
///
/// Returns a variant of [JsonPathParserError] if the parsing operation failed.
pub fn parse_json_path<T>(jp_str: &str) -> Result<JsonPath<T>, JsonPathParserError>
where
    T: JsonLike,
{
    JsonPathParser::parse(Rule::path, jp_str)
        .map_err(Box::new)?
        .next()
        .ok_or(JsonPathParserError::UnexpectedPestOutput)
        .and_then(parse_internal)
}

/// Internal function takes care of the logic by parsing the operators and unrolling the string into the final result.
///
/// # Errors
///
/// Returns a variant of [JsonPathParserError] if the parsing operation failed
fn parse_internal<T>(rule: Pair<'_, Rule>) -> Result<JsonPath<T>, JsonPathParserError>
where
    T: JsonLike,
{
    match rule.as_rule() {
        Rule::path => rule
            .into_inner()
            .next()
            .ok_or(JsonPathParserError::NoRulePath)
            .and_then(parse_internal),
        Rule::current => rule
            .into_inner()
            .next()
            .map(parse_internal)
            .unwrap_or(Ok(JsonPath::Empty))
            .map(Box::new)
            .map(JsonPath::Current),
        Rule::chain => rule
            .into_inner()
            .map(parse_internal)
            .collect::<Result<Vec<_>, _>>()
            .map(JsonPath::Chain),
        Rule::root => Ok(JsonPath::Root),
        Rule::wildcard => Ok(JsonPath::Wildcard),
        Rule::descent => parse_key(down(rule)?)?
            .map(JsonPath::Descent)
            .ok_or(JsonPathParserError::NoJsonPathDescent),
        Rule::descent_w => Ok(JsonPath::DescentW),
        Rule::function => Ok(JsonPath::Fn(Function::Length)),
        Rule::field => parse_key(down(rule)?)?
            .map(JsonPath::Field)
            .ok_or(JsonPathParserError::NoJsonPathField),
        Rule::index => parse_index(rule).map(JsonPath::Index),
        rule => Err(JsonPathParserError::InvalidTopLevelRule(rule)),
    }
}

/// parsing the rule 'key' with the structures either .key or .\['key'\]
fn parse_key(rule: Pair<Rule>) -> Result<Option<String>, JsonPathParserError> {
    let parsed_key = match rule.as_rule() {
        Rule::key | Rule::key_unlim | Rule::string_qt => parse_key(down(rule)?),
        Rule::key_lim | Rule::inner => Ok(Some(String::from(rule.as_str()))),
        _ => Ok(None),
    };
    parsed_key
}

fn parse_slice<T>(pairs: Pairs<Rule>) -> Result<JsonPathIndex<T>, JsonPathParserError> {
    let mut start = None;
    let mut end = None;
    let mut step = None;
    fn validate_min_0(val: &str) -> Result<(), JsonPathParserError> {
        if val == "-0" {
            Err(JsonPathParserError::InvalidJsonPath("-0 is not a valid value for a slice".to_string()))
        }else {
            Ok(())
        }
    }

    for in_pair in pairs {
        match in_pair.as_rule() {
            Rule::start_slice => {
                let parsed_val = in_pair.as_str().trim();
                validate_min_0(parsed_val)?;
                start = Some(parsed_val.parse::<i64>().map_err(|e| (e, parsed_val))?);
            }
            Rule::end_slice => {
                let parsed_val = in_pair.as_str().trim();
                validate_min_0(parsed_val)?;
                end = Some(parsed_val.parse::<i64>().map_err(|e| (e, parsed_val))?);
            }
            Rule::step_slice => {
                if let Some(parsed_val) = in_pair
                    .into_inner()
                    .next()
                    .map(|v| v.as_str().trim())
                {
                    validate_min_0(parsed_val)?;
                    step =Some(parsed_val.parse::<i64>().map_err(|e| (e, parsed_val))?);
                }


            }
            _ => (),
        }
    }
    Ok(JsonPathIndex::Slice(start, end, step))
}

fn parse_unit_keys<T>(pairs: Pairs<Rule>) -> Result<JsonPathIndex<T>, JsonPathParserError> {
    let mut keys = vec![];

    for pair in pairs {
        keys.push(String::from(down(pair)?.as_str()));
    }
    Ok(JsonPathIndex::UnionKeys(keys))
}

fn number_to_value<T>(number: &str) -> Result<T, JsonPathParserError>
where
    T: From<i64> + From<f64>,
{
    match number
        .parse::<i64>()
        .ok()
        .map(T::from)
        .or_else(|| number.parse::<f64>().ok().map(T::from))
    {
        Some(value) => Ok(value),
        None => Err(JsonPathParserError::InvalidNumber(number.to_string())),
    }
}

fn bool_to_value<T>(boolean: &str) -> T
where
    T: From<bool>,
{
    boolean
        .parse::<bool>()
        .map(T::from)
        .expect("unreachable: according to .pest this is either `true` or `false`")
}

fn parse_unit_indexes<T>(pairs: Pairs<Rule>) -> Result<JsonPathIndex<T>, JsonPathParserError>
where
    T: From<i64> + From<f64>,
{
    let mut keys = vec![];

    for pair in pairs {
        keys.push(number_to_value(pair.as_str())?);
    }
    Ok(JsonPathIndex::UnionIndex(keys))
}

fn parse_chain_in_operand<T>(rule: Pair<'_, Rule>) -> Result<Operand<T>, JsonPathParserError>
where
    T: JsonLike,
{
    let parsed_chain = match parse_internal::<T>(rule)? {
        JsonPath::Chain(elems) => {
            if elems.len() == 1 {
                match elems.first() {
                    Some(JsonPath::Index(JsonPathIndex::UnionKeys(keys))) => {
                        Operand::val(T::from(keys.clone()))
                    }
                    Some(JsonPath::Index(JsonPathIndex::UnionIndex(keys))) => {
                        Operand::val(T::from(keys.clone()))
                    }
                    Some(JsonPath::Field(f)) => Operand::val(T::from(vec![f.to_string()])),
                    _ => Operand::Dynamic(Box::new(JsonPath::Chain(elems))),
                }
            } else {
                Operand::Dynamic(Box::new(JsonPath::Chain(elems)))
            }
        }
        jp => Operand::Dynamic(Box::new(jp)),
    };
    Ok(parsed_chain)
}

fn parse_filter_index<T>(pair: Pair<'_, Rule>) -> Result<JsonPathIndex<T>, JsonPathParserError>
where
    T: JsonLike,
{
    Ok(JsonPathIndex::Filter(parse_logic_or(pair.into_inner())?))
}

fn parse_logic_or<T>(pairs: Pairs<'_, Rule>) -> Result<FilterExpression<T>, JsonPathParserError>
where
    T: JsonLike,
{
    let mut expr: Option<FilterExpression<T>> = None;
    // only possible for the loop not to produce any value (except Errors)
    if pairs.len() == 0 {
        return Err(JsonPathParserError::UnexpectedNoneLogicError(
            pairs.get_input().to_string(),
            pairs.as_str().to_string(),
        ));
    }
    for pair in pairs.into_iter() {
        let next_expr = parse_logic_and(pair.into_inner())?;
        match expr {
            None => expr = Some(next_expr),
            Some(e) => expr = Some(Or(Box::new(e), Box::new(next_expr))),
        }
    }
    Ok(expr.expect("unreachable: above len() == 0 check should have catched this"))
}

fn parse_logic_and<T>(pairs: Pairs<'_, Rule>) -> Result<FilterExpression<T>, JsonPathParserError>
where
    T: JsonLike,
{
    let mut expr: Option<FilterExpression<T>> = None;
    // only possible for the loop not to produce any value (except Errors)
    if pairs.len() == 0 {
        return Err(JsonPathParserError::UnexpectedNoneLogicError(
            pairs.get_input().to_string(),
            pairs.as_str().to_string(),
        ));
    }
    for pair in pairs {
        let next_expr = parse_logic_not(pair.into_inner())?;
        match expr {
            None => expr = Some(next_expr),
            Some(e) => expr = Some(And(Box::new(e), Box::new(next_expr))),
        }
    }
    Ok(expr.expect("unreachable: above len() == 0 check should have catched this"))
}

fn parse_logic_not<T>(
    mut pairs: Pairs<'_, Rule>,
) -> Result<FilterExpression<T>, JsonPathParserError>
where
    T: JsonLike,
{
    if let Some(rule) = pairs.peek().map(|x| x.as_rule()) {
        match rule {
            Rule::not => {
                pairs.next().expect("unreachable in arithmetic: should have a value as pairs.peek() was Some(_)");
                parse_logic_not(pairs)
                    .map(|expr|Not(Box::new(expr)))
            },
            Rule::logic_atom => parse_logic_atom(pairs.next().expect("unreachable in arithmetic: should have a value as pairs.peek() was Some(_)").into_inner()),
            rule => Err(JsonPathParserError::UnexpectedRuleLogicError(rule, pairs.get_input().to_string(), pairs.as_str().to_string())),
        }
    } else {
        Err(JsonPathParserError::UnexpectedNoneLogicError(
            pairs.get_input().to_string(),
            pairs.as_str().to_string(),
        ))
    }
}

fn parse_logic_atom<T>(
    mut pairs: Pairs<'_, Rule>,
) -> Result<FilterExpression<T>, JsonPathParserError>
where
    T: JsonLike,
{
    if let Some(rule) = pairs.peek().map(|x| x.as_rule()) {
        match rule {
            Rule::logic_or => parse_logic_or(pairs.next().expect("unreachable in arithmetic: should have a value as pairs.peek() was Some(_)").into_inner()),
            Rule::atom => {
                let left: Operand<T> = parse_atom(pairs.next().unwrap())?;
                if pairs.peek().is_none() {
                    Ok(FilterExpression::exists(left))
                } else {
                    let sign: FilterSign = FilterSign::new(pairs.next().expect("unreachable in arithmetic: should have a value as pairs.peek() was Some(_)").as_str());
                    let right: Operand<T> =
                        parse_atom(pairs.next().expect("unreachable in arithemetic: should have a right side operand"))?;
                    Ok(FilterExpression::Atom(left, sign, right))
                }
            }
            rule => Err(JsonPathParserError::UnexpectedRuleLogicError(rule, pairs.get_input().to_string(), pairs.as_str().to_string())),
        }
    } else {
        Err(JsonPathParserError::UnexpectedNoneLogicError(
            pairs.get_input().to_string(),
            pairs.as_str().to_string(),
        ))
    }
}

fn parse_atom<T>(rule: Pair<'_, Rule>) -> Result<Operand<T>, JsonPathParserError>
where
    T: JsonLike,
{
    let atom = down(rule.clone())?;
    let parsed_atom = match atom.as_rule() {
        Rule::number => Operand::Static(number_to_value(rule.as_str())?),
        Rule::string_qt => Operand::Static(T::from(down(atom)?.as_str())),
        Rule::chain => parse_chain_in_operand(down(rule)?)?,
        Rule::boolean => Operand::Static(bool_to_value(rule.as_str())),
        _ => Operand::Static(T::null()),
    };
    Ok(parsed_atom)
}

fn parse_index<T>(rule: Pair<'_, Rule>) -> Result<JsonPathIndex<T>, JsonPathParserError>
where
    T: JsonLike,
{
    let next = down(rule)?;
    let parsed_index = match next.as_rule() {
        Rule::unsigned => JsonPathIndex::Single(number_to_value(next.as_str())?),
        Rule::slice => parse_slice(next.into_inner())?,
        Rule::unit_indexes => parse_unit_indexes(next.into_inner())?,
        Rule::unit_keys => parse_unit_keys(next.into_inner())?,
        Rule::filter => parse_filter_index(down(next)?)?,
        _ => JsonPathIndex::Single(number_to_value(next.as_str())?),
    };
    Ok(parsed_index)
}

fn down(rule: Pair<Rule>) -> Result<Pair<Rule>, JsonPathParserError> {
    let error_message = rule.to_string();
    match rule.into_inner().next() {
        Some(rule) => Ok(rule),
        None => Err(JsonPathParserError::EmptyInner(error_message)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::macros::{chain, filter, idx, op};
    use crate::path;
    use serde_json::{json, Value};
    use std::panic;
    use crate::JsonPath::Index;
    use crate::parser::JsonPathIndex::Slice;

    fn test_failed(input: &str) {
        match parse_json_path::<Value>(input) {
            Ok(elem) => panic!("should be false but got {:?}", elem),
            Err(e) => println!("{}", e),
        }
    }

    fn test<T>(input: &str, expected: Vec<JsonPath<T>>)
    where
        T: JsonLike,
    {
        match parse_json_path::<T>(input) {
            Ok(JsonPath::Chain(elems)) => assert_eq!(elems, expected),
            Ok(e) => panic!("unexpected value {:?}", e),
            Err(e) => {
                panic!("parsing error {}", e);
            }
        }
    }

    #[test]
    fn path_test_extra(){
        test::<Value>("$ [ 'k' ]",
                      vec![ path!($),path!("k")
                      ]);
        test::<Value>("$..[ 'k']",
                      vec![ path!($),path!(.."k")
                      ]);
    }
    #[test]
    fn path_test() {

        test::<Value>("$.k.['k']['k']..k..['k'].*.[*][*][1][1,2]['k','k'][:][10:][:10][10:10:10][?(@)][?(@.abc >= 10)]",
             vec![
                 path!($),
                 path!("k"),
                 path!("k"),
                 path!("k"),
                 path!(.."k"),
                 path!(.."k"),
                 path!(*),
                 path!(*),
                 path!(*),
                 path!(idx!(1)),
                 path!(idx!(idx 1,2)),
                 path!(idx!("k","k")),
                 path!(idx!([; ;])),
                 path!(idx!([10; ;])),
                 path!(idx!([;10;])),
                 path!(idx!([10;10;10])),
                 path!(idx!(?filter!(op!(chain!(path!(@path!()))), "exists", op!(path!())))),
                 path!(idx!(?filter!(op!(chain!(path!(@,path!("abc")))), ">=", op!(10)))),
             ]);
        test::<Value>(
            "$..*[?(@.isbn)].title",
            vec![
                // Root, DescentW, Index(Filter(Atom(Dynamic(Chain([Current(Chain([Field("isbn")]))])), Exists, Dynamic(Empty)))), Field("title")
                path!($),
                path!(..*),
                path!(idx!(?filter!(op!(chain!(path!(@,path!("isbn")))), "exists", op!(path!())))),
                path!("title"),
            ],
        )
    }

    #[test]
    fn descent_test() {
        test::<Value>("..abc", vec![path!(.."abc")]);
        test::<Value>("..['abc']", vec![path!(.."abc")]);
        test_failed("...['abc']");
        test_failed("...abc");
    }

    #[test]
    fn field_test() {
        test::<Value>(".abc", vec![path!("abc")]);
        test::<Value>(".['abc']", vec![path!("abc")]);
        test::<Value>("['abc']", vec![path!("abc")]);
        test::<Value>(".['abc\\\"abc']", vec![path!("abc\\\"abc")]);
        test_failed(".abc()abc");
        test_failed("..[abc]");
        test_failed(".'abc'");
    }

    #[test]
    fn wildcard_test() {
        test::<Value>(".*", vec![path!(*)]);
        test::<Value>(".[*]", vec![path!(*)]);
        test::<Value>(".abc.*", vec![path!("abc"), path!(*)]);
        test::<Value>(".abc.[*]", vec![path!("abc"), path!(*)]);
        test::<Value>(".abc[*]", vec![path!("abc"), path!(*)]);
        test::<Value>("..*", vec![path!(..*)]);
        test_failed("abc*");
    }

    #[test]
    fn index_single_test() {
        test::<Value>("[1]", vec![path!(idx!(1))]);
        test_failed("[-1]");
        test_failed("[1a]");
    }

    #[test]
    fn index_slice_test() {
        test::<Value>("[1:1000:10]", vec![path!(idx!([1; 1000; 10]))]);
        test::<Value>("[:1000]", vec![path!(idx!([;1000;]))]);
        test::<Value>("[:]", vec![path!(idx!([;;]))]);
        test::<Value>("[::10]", vec![path!(idx!([;;10]))]);
        test_failed("[:::0]");
    }

    #[test]
    fn index_slice_symbols_test() {
        test::<Value>("[1:\r]", vec![Index(Slice(Some(1), None, None))]);
        test::<Value>("[1:1\r:2\t]", vec![Index(Slice(Some(1), Some(1), Some(2)))]);
        test::<Value>("[\n:1\r:1]", vec![Index(Slice(None, Some(1), Some(1)))]);
        test::<Value>("[1:2\r:2\n]", vec![Index(Slice(Some(1), Some(2), Some(2)))]);
    }

    #[test]
    fn index_union_test() {
        test::<Value>("[1,2,3]", vec![path!(idx!(idx 1,2,3))]);
        test::<Value>("['abc','bcd']", vec![path!(idx!("abc", "bcd"))]);
        test_failed("[]");
        test::<Value>("[-1,-2]", vec![path!(idx!(idx - 1, -2))]);
        test_failed("[abc,bcd]");
        test::<Value>("[\"abc\",\"bcd\"]", vec![path!(idx!("abc", "bcd"))]);
    }

    #[test]
    fn array_start_test() {
        test::<Value>(
            "$.[?(@.verb== \"TEST\")]",
            vec![
                path!($),
                path!(idx!(?filter!(op!(chain!(path!(@,path!("verb")))),"==",op!("TEST")))),
            ],
        );
    }

    #[test]
    fn logical_filter_test() {
        test::<Value>(
            "$.[?(@.verb == 'T' || @.size > 0 && @.size < 10)]",
            vec![
                path!($),
                path!(idx!(?
                filter!(
                    filter!(op!(chain!(path!(@,path!("verb")))), "==", op!("T")),
                    ||,
                    filter!(
                        filter!(op!(chain!(path!(@,path!("size")))), ">", op!(0)),
                        &&,
                        filter!(op!(chain!(path!(@,path!("size")))), "<", op!(10))
                    )
                ))),
            ],
        );
        test::<Value>(
            "$.[?((@.verb == 'T' || @.size > 0) && @.size < 10)]",
            vec![
                path!($),
                path!(idx!(?
                filter!(
                    filter!(
                       filter!(op!(chain!(path!(@,path!("verb")))), "==", op!("T")),
                        ||,
                        filter!(op!(chain!(path!(@,path!("size")))), ">", op!(0))
                    ),
                    &&,
                    filter!(op!(chain!(path!(@,path!("size")))), "<", op!(10))
                ))),
            ],
        );
        test::<Value>(
            "$.[?(@.verb == 'T' || @.size > 0 && @.size < 10 && @.elem == 0)]",
            vec![
                path!($),
                path!(idx!(?filter!(
                    filter!(op!(chain!(path!(@,path!("verb")))), "==", op!("T")),
                    ||,
                    filter!(
                        filter!(
                            filter!(op!(chain!(path!(@,path!("size")))), ">", op!(0)),
                            &&,
                            filter!(op!(chain!(path!(@,path!("size")))), "<", op!(10))
                        ),
                        &&,
                        filter!(op!(chain!(path!(@,path!("elem")))), "==", op!(0))
                    )

                ))),
            ],
        );
    }

    #[test]
    fn index_filter_test() {
        test::<Value>(
            "[?'abc' == 'abc']",
            vec![path!(idx!(?filter!(op!("abc"),"==",op!("abc") )))],
        );
        test::<Value>(
            "[?'abc' == 1]",
            vec![path!(idx!(?filter!( op!("abc"),"==",op!(1))))],
        );
        test::<Value>(
            "[?('abc' == true)]",
            vec![path!(idx!(?filter!( op!("abc"),"==",op!(true))))],
        );
        test::<Value>(
            "[?('abc' == null)]",
            vec![path!(
                idx!(?filter!( op!("abc"),"==",Operand::Static(Value::Null)))
            )],
        );

        test::<Value>(
            "[?(@.abc in ['abc','bcd'])]",
            vec![path!(
                idx!(?filter!(op!(chain!(path!(@,path!("abc")))),"in",Operand::val(json!(["abc","bcd"]))))
            )],
        );

        test::<Value>(
            "[?(@.abc.[*] in ['abc','bcd'])]",
            vec![path!(idx!(?filter!(
               op!(chain!(path!(@,path!("abc"), path!(*)))),
                "in",
                op!(s json!(["abc","bcd"]))
            )))],
        );
        test::<Value>(
            "[?(@.[*]..next in ['abc','bcd'])]",
            vec![path!(idx!(?filter!(
                op!(chain!(path!(@,path!(*), path!(.."next")))),
                "in",
                op!(s json!(["abc","bcd"]))
            )))],
        );

        test::<Value>(
            "[?(@[1] in ['abc','bcd'])]",
            vec![path!(idx!(?filter!(
                op!(chain!(path!(@,path!(idx!(1))))),
                "in",
                op!(s json!(["abc","bcd"]))
            )))],
        );
        test::<Value>(
            "[?(@ == 'abc')]",
            vec![path!(idx!(?filter!(
                op!(chain!(path!(@path!()))),"==",op!("abc")
            )))],
        );
        test::<Value>(
            "[?(@ subsetOf ['abc'])]",
            vec![path!(idx!(?filter!(
                op!(chain!(path!(@path!()))),"subsetOf",op!(s json!(["abc"]))
            )))],
        );
        test::<Value>(
            "[?(@[1] subsetOf ['abc','abc'])]",
            vec![path!(idx!(?filter!(
                op!(chain!(path!(@,path!(idx!(1))))),
                "subsetOf",
                op!(s json!(["abc","abc"]))
            )))],
        );
        test::<Value>(
            "[?(@ subsetOf [1,2,3])]",
            vec![path!(idx!(?filter!(
                op!(chain!(path!(@path!()))),"subsetOf",op!(s json!([1,2,3]))
            )))],
        );

        test_failed("[?(@[1] subsetof ['abc','abc'])]");
        test_failed("[?(@ >< ['abc','abc'])]");
        test_failed("[?(@ in {\"abc\":1})]");
    }

    #[test]
    fn fn_size_test() {
        test::<Value>(
            "$.k.length()",
            vec![path!($), path!("k"), JsonPath::Fn(Function::Length)],
        );

        test::<Value>(
            "$.k.length.field",
            vec![path!($), path!("k"), path!("length"), path!("field")],
        )
    }

    #[test]
    fn parser_error_test_invalid_rule() {
        let result = parse_json_path::<Value>("notapath");

        assert!(result.is_err());
        assert!(result
            .err()
            .unwrap()
            .to_string()
            .starts_with("Failed to parse rule"));
    }

    #[test]
    fn parser_error_test_empty_rule() {
        let result = parse_json_path::<Value>("");

        assert!(result.is_err());
        assert!(result
            .err()
            .unwrap()
            .to_string()
            .starts_with("Failed to parse rule"));
    }
}
