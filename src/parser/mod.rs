use serde_json::{Result, Value};

pub(crate) mod model;

pub fn parse(json: &str) -> Result<Value> {
    serde_json::from_str(json)
}

#[cfg(test)]
mod tests {
    use crate::parser::parse;

    #[test]
    fn parser_dummy_test() {
        let res = parse(r#"
        {
            "name": "John Doe",
            "age": 43,
            "phones": [
                "+44 1234567",
                "+44 2345678"
            ]
        }"#).unwrap();

        for (el, v) in res.as_object().unwrap() {
            println!("{} : {}", el, v)
        }
    }
}
