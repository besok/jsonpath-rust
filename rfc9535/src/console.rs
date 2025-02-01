use crate::suite::TestFailure;
use chrono::Local;
use colored::Colorize;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Error};
use std::io::Write;
pub fn process_results(results: Vec<TestResult>, skipped_cases: usize) -> Result<(), Error> {
    let (passed, failed): (Vec<_>, Vec<_>) = results.into_iter().partition(TestResult::is_ok);
    let total = passed.len() + failed.len() + skipped_cases;
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

    clean_file(5)?;

    println!(
        "\n{}:\n{}\n{}\n{}\n{}",
        format!("RFC9535 Compliance tests").underline().bold(),
        format!("Total: {}", total).bold(),
        format!("Passed: {}", passed_count).green().bold(),
        format!("Failed: {}", failed_count).red().bold(),
        format!("Skipped: {}", skipped_cases).bold()
    );
    Ok(())
}

fn clean_file(limit:usize) -> Result<(), Error> {
    let file_path = "test_suite/results.csv";
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;

    if lines.len() > limit {
        let header = &lines[0];
        let trimmed_lines = [&[header.clone()], &lines[lines.len() - limit..]].concat();

        let mut file = OpenOptions::new().write(true).truncate(true).open(file_path)?;
        for line in trimmed_lines {
            writeln!(file, "{}", line)?;
        }
    }

    Ok(())
}


pub type TestResult<'a> = Result<(), TestFailure<'a>>;
