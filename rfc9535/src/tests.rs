use jsonpath_rust::query::{js_path, Queried};
use jsonpath_rust::{JsonPath,  };
use serde_json::{json, Value};
use std::borrow::Cow;

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
            (&json!("ab"), "$[''a'']".to_string()).into(),
            (&json!("bc"), "$[''b'']".to_string()).into(),
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
        vec![(&json!("ab"), "$[''a''][''b'']".to_string()).into(),]
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
        vec![(&json!("bab"), "$['values'][2]".to_string()).into(),]
    );

    Ok(())
}
#[test]
fn name_sel() -> Queried<()> {
    let json = json!({
      "/": "A"
    });

    let vec = js_path("$['\\/']", &json)?;

    assert_eq!(vec, vec![(&json!("A"), "$[''\\/'']".to_string()).into(),]);

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

    let vec: Vec<String> = json
        .query_only_path("$[?@<3]")?
        .into_iter()
        .map(Option::unwrap_or_default)
        .collect();

    assert_eq!(vec, vec!["$['a']".to_string(), "$['b']".to_string()]);

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

    let vec: Vec<&Value> = json.query("$[?@.a==1E2]")?;
    assert_eq!(
        vec.iter().map(Cow::as_ref).collect::<Vec<_>>(),
        vec![&json!({"a":100, "d":"e"})]
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
        vec![(&json!("A"), "$['\"a\'\"']".to_string()).into(),]
    );

    Ok(())
}
#[test]
fn union() -> Queried<()> {
    let json = json!([0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

    let vec: Vec<QueryRes<Value>> = json.query_with_path("$[1,5:7]")?;
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

#[test]
fn basic_descendent() -> Queried<()> {
    let json = json!({
      "o": [
        0,
        1,
        [
          2,
          3
        ]
      ]
    });

    let vec = js_path("$..[1]", &json)?;
    assert_eq!(
        vec,
        vec![
            (&json!(1), "$['o'][1]".to_string()).into(),
            (&json!(3), "$['o'][2][1]".to_string()).into(),
        ]
    );

    Ok(())
}
#[test]
fn filter_absent() -> Queried<()> {
    let json = json!([
      {
        "list": [
          1
        ]
      }
    ]);

    let vec = js_path("$[?@.absent==@.list[9]]", &json)?;
    assert_eq!(
        vec,
        vec![(&json!({"list": [1]}), "$[0]".to_string()).into(),]
    );

    Ok(())
}

#[test]
fn filter_star() -> Queried<()> {
    let json = json!([1,[],[2],{},{"a": 3}]);

    let vec = json.query_with_path("$[?@.*]")?;
    assert_eq!(
        vec,
        vec![
            (&json!([2]), "$[2]".to_string()).into(),
            (&json!({"a": 3}), "$[4]".to_string()).into(),
        ]
    );

    Ok(())
}
