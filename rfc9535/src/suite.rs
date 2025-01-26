use colored::Colorize;
use jsonpath_rust::JsonPathParserError;
use serde_json::Value;

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
