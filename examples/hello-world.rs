use jsonpath_rust::JsonPath;
use serde_json::json;

fn main() {
    let data = json!({
        "Hello":"World",
        "Good":"Bye",
    });
    let path = JsonPath::try_from("$.Hello").unwrap();
    let search_result = path.find(&data);
    println!("Hello, {}", search_result);
}
