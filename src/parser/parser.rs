use pest::iterators::{Pair, Pairs};
use pest::{Parser};
use serde_json::{Value};
use crate::parser::model::{JsonPath, JsonPathIndex, Operand, FilterSign, FilterExpression, Function};
use pest::error::{Error};
use crate::parser::model::FilterExpression::{And, Or};

#[derive(Parser)]
#[grammar = "parser/grammar/json_path.pest"]
struct JsonPathParser;

/// the parsing function.
/// Since the parsing can finish with error the result is [[Result]]
pub fn parse_json_path(jp_str: &str) -> Result<JsonPath, Error<Rule>> {
    Ok(parse_internal(JsonPathParser::parse(Rule::path, jp_str)?.next().unwrap()))
}

/// Internal function takes care of the logic by parsing the operators and unrolling the string into the final result.
fn parse_internal(rule: Pair<Rule>) -> JsonPath {
    match rule.as_rule() {
        Rule::path => rule.into_inner().next().map(parse_internal).unwrap_or(JsonPath::Empty),
        Rule::current => JsonPath::Current(Box::new(rule.into_inner().next().map(parse_internal).unwrap_or(JsonPath::Empty))),
        Rule::chain => JsonPath::Chain(rule.into_inner().map(parse_internal).collect()),
        Rule::root => JsonPath::Root,
        Rule::wildcard => JsonPath::Wildcard,
        Rule::descent => parse_key(down(rule)).map(JsonPath::Descent).unwrap_or(JsonPath::Empty),
        Rule::function => JsonPath::Fn(Function::Length),
        Rule::field => parse_key(down(rule)).map(JsonPath::Field).unwrap_or(JsonPath::Empty),
        Rule::index => JsonPath::Index(parse_index(rule)),
        _ => JsonPath::Empty
    }
}

/// parsing the rule 'key' with the structures either .key or .]'key'[
fn parse_key(rule: Pair<Rule>) -> Option<String> {
    match rule.as_rule() {
        Rule::key
        | Rule::key_unlim
        | Rule::string_qt => parse_key(down(rule)),
        Rule::key_lim
        | Rule::inner => Some(String::from(rule.as_str())),
        _ => None
    }
}

fn parse_slice(mut pairs: Pairs<Rule>) -> JsonPathIndex {
    let mut start = 0;
    let mut end = 0;
    let mut step = 1;
    while pairs.peek().is_some() {
        let in_pair = pairs.next().unwrap();
        match in_pair.as_rule() {
            Rule::start_slice => start = in_pair.as_str().parse::<i32>().unwrap_or(start),
            Rule::end_slice => end = in_pair.as_str().parse::<i32>().unwrap_or(end),
            Rule::step_slice => step = down(in_pair).as_str().parse::<usize>().unwrap_or(step),
            _ => ()
        }
    }
    JsonPathIndex::Slice(start, end, step)
}

fn parse_unit_keys(mut pairs: Pairs<Rule>) -> JsonPathIndex {
    let mut keys = vec![];

    while pairs.peek().is_some() {
        keys.push(String::from(down(pairs.next().unwrap()).as_str()));
    }
    JsonPathIndex::UnionKeys(keys)
}

fn number_to_value(number: &str) -> Value {
    number.parse::<i64>().ok().map(Value::from)
        .or_else(|| number.parse::<f64>().ok().map(Value::from))
        .unwrap()
}

fn parse_unit_indexes(mut pairs: Pairs<Rule>) -> JsonPathIndex {
    let mut keys = vec![];

    while pairs.peek().is_some() {
        keys.push(number_to_value(pairs.next().unwrap().as_str()));
    }
    JsonPathIndex::UnionIndex(keys)
}

fn parse_chain_in_operand(rule: Pair<Rule>) -> Operand {
    match parse_internal(rule) {
        JsonPath::Chain(elems) => {
            if elems.len() == 1 {
                match elems.first() {
                    Some(JsonPath::Index(JsonPathIndex::UnionKeys(keys))) => Operand::val(Value::from(keys.clone())),
                    Some(JsonPath::Index(JsonPathIndex::UnionIndex(keys))) => Operand::val(Value::from(keys.clone())),
                    Some(JsonPath::Field(f)) => Operand::val(Value::Array(vec![Value::from(f.clone())])),
                    _ => Operand::Dynamic(Box::new(JsonPath::Chain(elems)))
                }
            } else {
                Operand::Dynamic(Box::new(JsonPath::Chain(elems)))
            }
        }
        jp => Operand::Dynamic(Box::new(jp))
    }
}


fn parse_filter_index( pair: Pair<Rule>) -> JsonPathIndex {
    JsonPathIndex::Filter(parse_logic(pair.into_inner()))
}

fn parse_logic(mut pairs: Pairs<Rule>) -> FilterExpression {
    let mut expr: Option<FilterExpression> = None;
    while pairs.peek().is_some() {
        let next_expr = parse_logic_and(pairs.next().unwrap().into_inner());
        match expr {
            None => expr = Some(next_expr),
            Some(e) => expr = Some(Or(Box::new(e), Box::new(next_expr)))
        }
    }
    expr.unwrap()
}

fn parse_logic_and(mut pairs: Pairs<Rule>) -> FilterExpression {
    let mut expr: Option<FilterExpression> = None;

    while pairs.peek().is_some() {
        let next_expr = parse_logic_atom(pairs.next().unwrap().into_inner());
        match expr {
            None => expr = Some(next_expr),
            Some(e) => expr = Some(And(Box::new(e), Box::new(next_expr)))
        }
    }
    expr.unwrap()
}

fn parse_logic_atom(mut pairs: Pairs<Rule>) -> FilterExpression {
    match pairs.peek().map(|x| x.as_rule()) {
        Some(Rule::logic) => parse_logic(pairs.next().unwrap().into_inner()),
        Some(Rule::atom) => {
            let left: Operand = parse_atom(pairs.next().unwrap());
            if pairs.peek().is_none() {
                FilterExpression::exists(left)
            } else {
                let sign: FilterSign = FilterSign::new(pairs.next().unwrap().as_str());
                let right: Operand = parse_atom(pairs.next().unwrap());
                FilterExpression::Atom(left, sign, right)
            }
        }
        Some(x) => panic!("unexpected => {:?}", x),
        None => panic!("unexpected none")
    }
}

fn parse_atom(rule: Pair<Rule>) -> Operand {
    let atom = down(rule.clone());
    match atom.as_rule() {
        Rule::number => Operand::Static(number_to_value(rule.as_str())),
        Rule::string_qt => Operand::Static(Value::from(down(atom).as_str())),
        Rule::chain => parse_chain_in_operand(down(rule)),
        Rule::boolean => Operand::Static(rule.as_str().parse().unwrap()),
        _ => Operand::Static(Value::Null)
    }
}


fn parse_index(rule: Pair<Rule>) -> JsonPathIndex {
    let next = down(rule);
    match next.as_rule() {
        Rule::unsigned => JsonPathIndex::Single(number_to_value(next.as_str())),
        Rule::slice => parse_slice(next.into_inner()),
        Rule::unit_indexes => parse_unit_indexes(next.into_inner()),
        Rule::unit_keys => parse_unit_keys(next.into_inner()),
        Rule::filter => parse_filter_index(down(next)),
        _ => JsonPathIndex::Single(number_to_value(next.as_str()))
    }
}

fn down(rule: Pair<Rule>) -> Pair<Rule> {
    rule.into_inner().next().unwrap()
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::panic;
    use serde_json::{json};
    use crate::{filter, idx, op, path, chain, function};

    fn test_failed(input: &str) {
        match parse_json_path(input) {
            Ok(elem) => panic!("should be false but got {:?}", elem),
            Err(e) => println!("{}", e)
        }
    }

    fn test(input: &str, expected: Vec<JsonPath>) {
        match parse_json_path(input) {
            Ok(JsonPath::Chain(elems)) => assert_eq!(elems, expected),
            Ok(e) => panic!("unexpected value {:?}", e),
            Err(e) => {
                panic!("parsing error {}",e);
            }
        }
    }

    #[test]
    fn path_test() {
        test("$.k.['k']['k']..k..['k'].*.[*][*][1][1,2]['k','k'][:][10:][:10][10:10:10][?(@)][?(@.abc >= 10)]",
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
             ])
    }

    #[test]
    fn descent_test() {
        test("..abc", vec![path!(.."abc")]);
        test("..['abc']", vec![path!(.."abc")]);
        test_failed("...['abc']");
        test_failed("...abc");
    }

    #[test]
    fn field_test() {
        test(".abc", vec![path!("abc")]);
        test(".['abc']", vec![path!("abc")]);
        test("['abc']", vec![path!("abc")]);
        test(".['abc\\\"abc']", vec![path!("abc\\\"abc")]);
        test_failed(".abc()abc");
        test_failed("..[abc]");
        test_failed(".'abc'");
    }

    #[test]
    fn wildcard_test() {
        test(".*", vec![path!(*)]);
        test(".[*]", vec![path!(*)]);
        test(".abc.*", vec![path!("abc"), path!(*)]);
        test(".abc.[*]", vec![path!("abc"), path!(*)]);
        test(".abc[*]", vec![path!("abc"), path!(*)]);
        test_failed("..*");
        test_failed("abc*");
    }

    #[test]
    fn index_single_test() {
        test("[1]", vec![path!(idx!(1))]);
        test_failed("[-1]");
        test_failed("[1a]");
    }

    #[test]
    fn index_slice_test() {
        test("[1:1000:10]", vec![path!(idx!([1; 1000; 10]))]);
        test("[:1000:10]", vec![path!(idx!([0; 1000; 10]))]);
        test("[:1000]", vec![path!(idx!([;1000;]))]);
        test("[:]", vec![path!(idx!([;;]))]);
        test("[::10]", vec![path!(idx!([;;10]))]);
        test_failed("[::-1]");
        test_failed("[:::0]");
    }

    #[test]
    fn index_union_test() {
        test("[1,2,3]", vec![path!(idx!(idx 1,2,3))]);
        test("['abc','bcd']", vec![path!(idx!("abc","bcd"))]);
        test_failed("[]");
        test("[-1,-2]", vec![path!(idx!(idx -1, -2))]);
        test_failed("[abc,bcd]");
        test_failed("[\"abc\",\"bcd\"]");
    }

    #[test]
    fn array_start_test() {
        test("$.[?(@.verb== 'TEST')]", vec![
            path!($),
            path!(idx!(?filter!(op!(chain!(path!(@,path!("verb")))),"==",op!("TEST"))))]);
    }

    #[test]
    fn logical_filter_test() {
        test("$.[?(@.verb == 'T' || @.size > 0 && @.size < 10)]", vec![
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
                )))
        ]);
        test("$.[?((@.verb == 'T' || @.size > 0) && @.size < 10)]", vec![
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
                )))
        ]);
        test("$.[?(@.verb == 'T' || @.size > 0 && @.size < 10 && @.elem == 0)]", vec![
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

                )))
        ]);
    }

    #[test]
    fn index_filter_test() {
        test("[?('abc' == 'abc')]", vec![path!(idx!(?filter!(op!("abc"),"==",op!("abc") )))]);
        test("[?('abc' == 1)]", vec![path!(idx!(?filter!( op!("abc"),"==",op!(1))))]);
        test("[?('abc' == true)]", vec![path!(idx!(?filter!( op!("abc"),"==",op!(true))))]);
        test("[?('abc' == null)]", vec![path!(idx!(?filter!( op!("abc"),"==",Operand::Static(Value::Null))))]);

        test("[?(@.abc in ['abc','bcd'])]", vec![
            path!(
                idx!(?filter!(op!(chain!(path!(@,path!("abc")))),"in",Operand::val(json!(["abc","bcd"]))))
            )
        ]);

        test("[?(@.abc.[*] in ['abc','bcd'])]", vec![path!(idx!(?filter!(
           op!(chain!(path!(@,path!("abc"), path!(*)))),
            "in",
            op!(s json!(["abc","bcd"]))
        )))]);
        test("[?(@.[*]..next in ['abc','bcd'])]", vec![path!(idx!(?filter!(
            op!(chain!(path!(@,path!(*), path!(.."next")))),
            "in",
            op!(s json!(["abc","bcd"]))
        )))]);

        test("[?(@[1] in ['abc','bcd'])]", vec![path!(idx!(?filter!(
            op!(chain!(path!(@,path!(idx!(1))))),
            "in",
            op!(s json!(["abc","bcd"]))
        )))]);
        test("[?(@ == 'abc')]", vec![path!(idx!(?filter!(
            op!(chain!(path!(@path!()))),"==",op!("abc")
        )))]);
        test("[?(@ subsetOf ['abc'])]", vec![path!(idx!(?filter!(
            op!(chain!(path!(@path!()))),"subsetOf",op!(s json!(["abc"]))
        )))]);
        test("[?(@[1] subsetOf ['abc','abc'])]", vec![path!(idx!(?filter!(
            op!(chain!(path!(@,path!(idx!(1))))),
            "subsetOf",
            op!(s json!(["abc","abc"]))
        )))]);
        test("[?(@ subsetOf [1,2,3])]", vec![path!(idx!(?filter!(
            op!(chain!(path!(@path!()))),"subsetOf",op!(s json!([1,2,3]))
        )))]);

        test_failed("[?(@[1] subsetof ['abc','abc'])]");
        test_failed("[?(@ >< ['abc','abc'])]");
        test_failed("[?(@ in {\"abc\":1})]");
    }

    #[test]
    fn fn_size_test() {
        test("$.k.length()",
             vec![
                 path!($),
                 path!("k"),
                 function!(length)
             ]);

        test("$.k.length.field",
             vec![
                 path!($),
                 path!("k"),
                 path!("length"),
                 path!("field"),
             ])

    }
}