use jsonpath_rust::JsonPathInst;
use serde_json::json;
use std::str::FromStr;

fn main() {
    let data = json!({
        "Hello":"World",
        "Good":"Bye",
    });
    let path = JsonPathInst::from_str("$.Hello").unwrap();
    let search_result = jsonpath_rust::find(&path, &data);
    println!("Hello, {}", search_result);
}
