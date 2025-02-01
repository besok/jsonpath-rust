use colored::Colorize;
use jsonpath_rust::{JsonPath, JsonPathParserError};
use serde_json::Value;
use std::str::FromStr;
use crate::console::TestResult;

type SkippedCases = usize;

pub fn get_suite() -> Result<(Vec<TestCase>, SkippedCases), std::io::Error> {
    let file = std::fs::File::open("test_suite/jsonpath-compliance-test-suite/cts.json")?;
    let suite: TestCases = serde_json::from_reader(std::io::BufReader::new(file))?;
    let suite: Vec<TestCase> = suite.tests;

    let filter = std::fs::File::open("test_suite/filtered_cases.json")?;
    let filter: Vec<FilterCase> = serde_json::from_reader(std::io::BufReader::new(filter))?;
    let mut skipped_cases = 0;
    Ok((
        suite
            .into_iter()
            .filter(|case| {
                if let Some(f) = filter.iter().find(|filter| case.name == filter.name) {
                    println!(
                        "Skipping test case: `{}` because of reason: `{}`",
                        case.name.green(),
                        f.reason.green()
                    );
                    skipped_cases += 1;
                    false
                } else {
                    true
                }
            })
            .collect(),
        skipped_cases,
    ))
}
pub fn handle_test_case(case: &TestCase) -> TestResult {
    let js_path: Result<JsonPath<Value>, _> = JsonPath::from_str(case.selector.as_str());

    if case.invalid_selector {
        if js_path.is_ok() {
            Err(TestFailure::invalid(case))
        } else {
            Ok(())
        }
    } else {
        if let Some(doc) = case.document.as_ref() {
            let js_path = js_path.map_err(|err| (err, case))?;
            let result = js_path.find(doc);

            match (case.result.as_ref(), case.results.as_ref()) {
                (Some(expected), _) => {
                    if result == *expected {
                        Ok(())
                    } else {
                        Err(TestFailure::match_one(case, &result))
                    }
                }
                (None, Some(expected)) => {
                    if expected.iter().any(|exp| result == *exp) {
                        Ok(())
                    } else {
                        Err(TestFailure::match_any(case, &result))
                    }
                }
                _ => Ok(()),
            }
        } else {
            Ok(())
        }
    }
}


#[derive(serde::Deserialize)]
struct FilterCase {
    name: String,
    reason: String,
}

#[derive(serde::Deserialize)]
pub struct TestCase {
    pub(crate) name: String,
    pub(crate) selector: String,
    pub(crate) document: Option<Value>,
    pub(crate) result: Option<Value>,
    pub(crate) results: Option<Vec<Value>>,
    #[serde(default)]
    pub(crate) invalid_selector: bool,
}
#[derive(serde::Deserialize)]
pub struct TestCases {
    pub(crate) description: String,
    pub(crate) tests: Vec<TestCase>,
}

pub struct TestFailure<'a>(pub &'a TestCase, pub String);

impl<'a> From<(JsonPathParserError, &'a TestCase)> for TestFailure<'a> {
    fn from((err, case): (JsonPathParserError, &'a TestCase)) -> Self {
        TestFailure(case, format!("Error parsing path: {}", err))
    }
}

impl<'a> TestFailure<'a> {
    pub(crate) fn invalid(case: &'a TestCase) -> Self {
        TestFailure(
            case,
            format!(
                "The path should have been considered invalid: {}",
                case.selector
            ),
        )
    }

    pub(crate) fn match_one(case: &'a TestCase, actual: &Value) -> Self {
        TestFailure(
            case,
            format!(
                "Actual did not match expected. Actual: {:?}, Expected: {:?}",
                actual, &case.result
            ),
        )
    }
    pub(crate) fn match_any(case: &'a TestCase, actual: &Value) -> Self {
        TestFailure(
            case,
            format!(
                "Actual did not match expected. Actual: {:?}, Expected: {:?}",
                actual, &case.results
            ),
        )
    }
}

