use jsonpath_rust::query::{js_path, Queried};
use jsonpath_rust::{JsonPathParserError, JsonPathQuery};
use serde_json::json;

#[test]
fn slice_selector_zero_step() -> Result<(), JsonPathParserError> {
    assert_eq!(
        json!([0, 1, 2, 3, 4, 5, 6, 7, 8, 9]).path("$[1:2:0]")?,
        json!([])
    );
    Ok(())
}

#[test]
fn slice_selector_with_neg_step() -> Result<(), JsonPathParserError> {
    assert_eq!(json!([]).path("$[::-1]")?, json!([]));
    Ok(())
}

#[test]
fn slice_selector_with_max() -> Result<(), JsonPathParserError> {
    assert!(json!([]).path("$[:9007199254740992:1]").is_err());
    Ok(())
}
#[test]
fn slice_selector_with_max_plus_1() -> Result<(), JsonPathParserError> {
    assert!(json!([]).path("$[::9007199254740992]").is_err());
    Ok(())
}
#[test]
fn exclude_embedded_character() -> Result<(), JsonPathParserError> {
    assert!(json!([]).path("$[\"\"]").is_err());
    Ok(())
}
#[test]
fn slice_selector_leading_m0() -> Result<(), JsonPathParserError> {
    assert!(json!([]).path("$[-0::]").is_err());
    Ok(())
}
#[test]
fn slice_selector_with_last() -> Result<(), JsonPathParserError> {
    assert_eq!(json!([1, 2, 3, 4, 5, 6]).path("$[1:5\r:2]")?, json!([2, 4]));
    Ok(())
}
#[test]
fn extra_symbols() -> Result<(), JsonPathParserError> {
    assert_eq!(json!({"a": "ab"}).path("$['a'\r]")?, json!(["ab"]));
    Ok(())
}

#[test]
fn filter() -> Result<(), JsonPathParserError> {
    assert_eq!(json!({"a": 1,"b": null}).path("$[?@]")?, json!([1, null]));
    Ok(())
}
#[test]
fn filter_quoted_lit() -> Result<(), JsonPathParserError> {
    assert_eq!(
        json!(["quoted' literal", "a", "quoted\\' literal"])
            .path("$[?@ == \"quoted' literal\"]")?,
        json!(["quoted' literal"])
    );
    Ok(())
}

#[test]
fn invalid_esc_single_q() -> Result<(), JsonPathParserError> {
    assert!(json!([]).path("$['\\\"']").is_err());
    Ok(())
}

#[test]
fn index_neg() -> Result<(), JsonPathParserError> {
    assert_eq!(json!([]).path("$[-9007199254740991]")?, json!([]));
    Ok(())
}
#[test]
fn field_num() -> Result<(), JsonPathParserError> {
    assert!(json!([]).path("$.1").is_err());
    Ok(())
}
#[test]
fn field_surrogate_pair() -> Result<(), JsonPathParserError> {
    assert !(json!([]).path("$['\\uD834\\uDD1E']").is_err());
    Ok(())
}

#[test]
fn union_quotes() -> Queried<()> {
    let json = json!({
      "a": "ab",
      "b": "bc"
    });

    let vec = js_path("$['a',\r'b']", &json)?;

    assert_eq!(
        vec,
        vec![
            (&json!("ab"), "$.[''a'']".to_string()).into(),
            (&json!("bc"), "$.[''b'']".to_string()).into(),
        ]
    );

    Ok(())
}

#[test]
fn space_between_selectors() -> Queried<()> {
    let json = json!({
      "a": {
        "b": "ab"
      }
    });

    let vec = js_path("$['a'] \r['b']", &json)?;

    assert_eq!(
        vec,
        vec![(&json!("ab"), "$.[''a''].[''b'']".to_string()).into(),]
    );

    Ok(())
}
#[test]
fn space_in_search() -> Queried<()> {
    let json = json!(["foo", "123"]);

    let vec = js_path("$[?search(@\n,'[a-z]+')]", &json)?;

    assert_eq!(vec, vec![(&json!("foo"), "$[0]".to_string()).into(),]);

    Ok(())
}
#[test]
fn filter_key() -> Queried<()> {
    let json = json!([
      {
        "a": "b",
        "d": "e"
      },
      {
        "a": 1,
        "d": "f"
      }
    ]);

    let vec = js_path("$[?@.a!=\"b\"]", &json)?;

    assert_eq!(
        vec,
        vec![(&json!({"a":1, "d":"f"}), "$[1]".to_string()).into(),]
    );

    Ok(())
}

#[test]
fn regex_key() -> Queried<()> {
    let json = json!({
      "regex": "b.?b",
      "values": [
        "abc",
        "bcd",
        "bab",
        "bba",
        "bbab",
        "b",
        true,
        [],
        {}
      ]
    });

    let vec = js_path("$.values[?match(@, $.regex)]", &json)?;

    assert_eq!(
        vec,
        vec![(&json!("bab"), "$.['values'][2]".to_string()).into(),]
    );

    Ok(())
}
#[test]
fn name_sel() -> Queried<()> {
    let json = json!({
      "/": "A"
    });

    let vec = js_path("$['\\/']", &json)?;

    assert_eq!(vec, vec![(&json!("A"), "$.[''\\/'']".to_string()).into(),]);

    Ok(())
}
#[test]
fn unicode_fns() -> Queried<()> {
    let json = json!(["ж", "Ж", "1", "жЖ", true, [], {}]);

    let vec = js_path("$[?match(@, '\\\\p{Lu}')]", &json)?;

    assert_eq!(vec, vec![(&json!("Ж"), "$[1]".to_string()).into(),]);

    Ok(())
}
#[test]
fn fn_res_can_not_compare() -> Queried<()> {
    let json = json!({});

    let vec = js_path("$[?match(@.a, 'a.*')==true]", &json);

    assert!(vec.is_err());

    Ok(())
}
#[test]
fn too_small() -> Queried<()> {
    let json = json!({});

    let vec = js_path("$[-9007199254740992]", &json);

    assert!(vec.is_err());

    Ok(())
}
#[test]
fn filter_data() -> Queried<()> {
    let json = json!({
      "a": 1,
      "b": 2,
      "c": 3
    });

    let vec = js_path("$[?@<3]", &json)?;

    assert_eq!(
        vec,
        vec![
            (&json!(1), "$.['a']".to_string()).into(),
            (&json!(2), "$.['b']".to_string()).into(),
        ]
    );

    Ok(())
}
#[test]
fn exp_no_error() -> Queried<()> {
    let json = json!([
        {
          "a": 100,
          "d": "e"
        },
        {
          "a": 100.1,
          "d": "f"
        },
        {
          "a": "100",
          "d": "g"
        }
      ]);

    let vec = js_path("$[?@.a==1E2]", &json)?;
    assert_eq!(
        vec,
        vec![(&json!({"a":100, "d":"e"}), "$[0]".to_string()).into(),]
    );

    Ok(())
}
#[test]
fn single_quote() -> Queried<()> {
    let json = json!({
        "a'": "A",
        "b": "B"
      });

    let vec = js_path("$[\"a'\"]", &json)?;
    assert_eq!(
        vec,
        vec![(&json!("A"), "$.['\"a\'\"']".to_string()).into(),]
    );

    Ok(())
}
#[test]
fn union() -> Queried<()> {
    let json = json!([
        0,
        1,
        2,
        3,
        4,
        5,
        6,
        7,
        8,
        9
      ]);

    let vec = js_path("$[1,5:7]", &json)?;
    assert_eq!(
        vec,
        vec![
            (&json!(1), "$[1]".to_string()).into(),
            (&json!(5), "$[5]".to_string()).into(),
            (&json!(6), "$[6]".to_string()).into(),
        ]
    );

    Ok(())
}

