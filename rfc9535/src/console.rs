use crate::suite::TestFailure;
use chrono::Local;
use colored::Colorize;
use std::fs::OpenOptions;
use std::io::Error;
use std::io::Write;
pub fn process_results(results: Vec<TestResult>) -> Result<(), Error> {
    let (passed, failed): (Vec<_>, Vec<_>) = results.into_iter().partition(TestResult::is_ok);
    let total = passed.len() + failed.len();
    let passed_count = passed.len();
    let failed_count = failed.len();
    let date = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    if failed_count > 0 {
        println!("\n{}:", "Failed tests".bold());
        println!("\n");
    }
    for failure in failed.iter() {
        if let Err(TestFailure(case, reason)) = failure {
            println!(" ------- {} -------", case.name.bold());
            println!("{}", reason.bold().red());
        }
    }



    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("test_suite/results.csv")?;
    writeln!(
        file,
        "{}; {}; {}; {}",
        total, passed_count, failed_count, date
    )?;
    println!(
        "\n{}:\n{}\n{}\n{}",
        format!("RFC9535 Compliance tests").underline().bold(),
        format!("Total: {}", total).bold(),
        format!("Passed: {}", passed_count).green().bold(),
        format!("Failed: {}", failed_count).red().bold()
    );
    Ok(())
}

pub type TestResult<'a> = Result<(), TestFailure<'a>>;

