use std::io::{BufReader, Error};
use std::str::FromStr;
use colored::Colorize;
use serde_json::Value;
use jsonpath_rust::{JsonPath, JsonPathParserError};

fn main() -> Result<(), Error>{
    let file = std::fs::File::open("test_suite/rfc9535-cts.json")?;
    let suite: Vec<TestCase> = serde_json::from_reader(BufReader::new(file))?;
    let results = suite.iter().map(handle_test_case).collect::<Vec<TestResult>>();

    let (passed, failed): (Vec<_>, Vec<_>) = results.into_iter().partition(TestResult::is_ok);


    for failure in failed.iter() {
        if let Err(TestFailure(case, reason)) = failure {
            println!(" ------- {} -------", case.name.bold());
            println!("{}", reason.bold());
        }
    }

    println!(
        "\n{}:\n{}\n{}\n{}",
        format!("RFC9535 Compliance tests").underline().bold(),
        format!("Total: {}", passed.len() + failed.len()).bold(),
        format!("Passed: {}", passed.len()).green().bold(),
        format!("Failed: {}", failed.len()).red().bold()
    );


    Ok(())
}

type TestResult<'a> = Result<(), TestFailure<'a>>;
fn handle_test_case(case: &TestCase) -> TestResult {
    let js_path: Result<JsonPath<Value>, _> = JsonPath::from_str(case.selector.as_str());

    if case.invalid_selector {
        if js_path.is_ok() {
            Err(TestFailure::invalid(case))
        } else {
            Ok(())
        }
    } else {
        if let Some(doc) = case.document.as_ref(){

            let js_path = js_path.map_err(|err| (err, case))?;
            let result = js_path.find(doc);


            match (case.result.as_ref(), case.results.as_ref()) {
                (Some(expected), _) => {
                    if result == *expected {
                        Ok(())
                    } else {
                        Err(TestFailure::match_one(case, &result))
                    }
                },
                (None, Some(expected)) => {
                    if expected.iter().any(|exp| result == *exp) {
                        Ok(())
                    } else {
                        Err(TestFailure::match_any(case , &result))
                    }
                },
                _ => Ok(())
            }


        } else {
            Ok(())
        }

    }
}



#[derive(serde::Deserialize)]
struct TestCase {
    name: String,
    selector: String,
    document: Option<Value>,
    result: Option<Value>,
    results: Option<Vec<Value>>,
    #[serde(default)]
    invalid_selector: bool,
}

struct TestFailure<'a>(&'a TestCase , String);

impl<'a> From<(JsonPathParserError, &'a TestCase)> for TestFailure<'a> {
    fn from((err, case): (JsonPathParserError, &'a TestCase)) -> Self {
        TestFailure(case, format!("Error parsing path: {}", err))
    }
}

impl<'a> TestFailure<'a> {

    fn invalid(case: &'a TestCase) -> Self {
        TestFailure(case, format!("The path should have been considered invalid: {}", case.selector))
    }

    fn match_one(case: &'a TestCase, actual:&Value) -> Self {
        TestFailure(case, format!("Actual did not match expected. Actual: {:?}, Expected: {:?}", actual, &case.result))
    }
    fn match_any(case: &'a TestCase,actual:&Value) -> Self {
        TestFailure(case, format!("Actual did not match expected. Actual: {:?}, Expected: {:?}", actual, &case.results))
    }

}

// this test caused the bug
// {
//     "name": "slice selector, zero step",
//     "selector": "$[1:2:0]",
//     "document": [
//       0,
//       1,
//       2,
//       3,
//       4,
//       5,
//       6,
//       7,
//       8,
//       9
//     ],
//     "result": [],
//     "tags": [
//       "slice"
//     ]
//   },
//