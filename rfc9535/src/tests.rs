use serde_json::json;
use jsonpath_rust::{JsonPathParserError, JsonPathQuery};


#[test]
fn slice_selector_zero_step() -> Result<(),JsonPathParserError> {
    assert_eq!(json!([0, 1, 2, 3, 4, 5, 6, 7, 8, 9]).path("$[1:2:0]")?, json!([]));
    Ok(())
}

#[test]
fn slice_selector_with_neg_step() -> Result<(),JsonPathParserError> {
    assert_eq!(json!([]).path("$[::-1]")?, json!([]));
    Ok(())
}

#[test]
fn slice_selector_with_max() -> Result<(),JsonPathParserError> {
    assert!(json!([]).path("$[:9007199254740992:1]").is_err());
    Ok(())
}
#[test]
fn slice_selector_with_max_plus_1() -> Result<(),JsonPathParserError> {
    assert!(json!([]).path("$[::9007199254740992]").is_err());
    Ok(())
}
#[test]
fn exclude_embedded_character() -> Result<(),JsonPathParserError> {
    assert!(json!([]).path("$[\"\"]").is_err());
    Ok(())
}
#[test]
fn slice_selector_leading_m0() -> Result<(),JsonPathParserError> {
    assert!(json!([]).path("$[-0::]").is_err());
    Ok(())
}
#[test]
fn slice_selector_with_last() -> Result<(),JsonPathParserError> {
    assert_eq!(json!([1, 2, 3, 4, 5, 6]).path("$[1:5\r:2]")?, json!([2,4]));
    Ok(())
}
#[test]
fn extra_symbols() -> Result<(),JsonPathParserError> {
    assert_eq!(json!({"a": "ab"}).path("$['a'\r]")?, json!([ "ab"]));
    Ok(())
}

#[test]
fn filter() -> Result<(),JsonPathParserError> {
    assert_eq!(json!({"a": 1,"b": null}).path("$[?@]")?, json!([ 1, null]));
    Ok(())
}
#[test]
fn filter_quoted_lit() -> Result<(),JsonPathParserError> {
    assert_eq!(json!([
        "quoted' literal",
        "a",
        "quoted\\' literal"
      ]).path("$[?@ == \"quoted' literal\"]")?, json!(["quoted' literal"]));
    Ok(())
}

#[test]
fn invalid_esc_single_q() -> Result<(),JsonPathParserError> {
    assert!(json!([]).path("$['\\\"']").is_err());
    Ok(())
}

#[test]
fn index_neg() -> Result<(),JsonPathParserError> {
    assert_eq!(json!([]).path("$[-9007199254740991]")?, json!([]) );
    Ok(())
}