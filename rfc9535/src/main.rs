
mod console;
mod suite;
mod tests;
use crate::suite::get_suite;
use colored::Colorize;
use console::TestResult;
use std::io::Error;
use std::io::Write;
use std::str::FromStr;

fn main() -> Result<(), Error> {
    let (cases, skipped) = get_suite()?;
    console::process_results(
        cases
            .iter()
            .map(suite::handle_test_case)
            .collect::<Vec<TestResult>>(),
        skipped,
    )
}

