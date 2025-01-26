mod console;
mod suite;
mod tests;

use crate::suite::{get_suite, TestCase, TestFailure};
use colored::Colorize;
use console::TestResult;
use jsonpath_rust::JsonPath;
use serde_json::Value;
use std::io::Error;
use std::io::Write;
use std::str::FromStr;

fn main() -> Result<(), Error> {
    let (cases, skipped) = get_suite()?;
    console::process_results(
        cases
            .iter()
            .map(handle_test_case)
            .collect::<Vec<TestResult>>(),
        skipped,
    )
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
