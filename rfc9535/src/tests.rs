use serde_json::json;
use jsonpath_rust::{JsonPathParserError, JsonPathQuery};


#[test]
fn slice_selector_zero_step() -> Result<(),JsonPathParserError> {
    assert_eq!(json!([0, 1, 2, 3, 4, 5, 6, 7, 8, 9]).path("$[1:2:0]")?, json!([]));
    Ok(())
}