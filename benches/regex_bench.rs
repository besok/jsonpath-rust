use std::str::FromStr;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use once_cell::sync::Lazy;
use serde_json::{json, Value};
use jsonpath_rust::{CONFIG, JsonPathFinder, JsonPathInst, JsonPathQuery};
use jsonpath_rust::path::config::cache::{DefaultRegexCacheInst, RegexCache};
use jsonpath_rust::path::config::JsonPathConfig;



fn regex_perf_test_after() {
    let json = Box::new(json!({
            "author":"abcd(Rees)",
        }));

    let _v = (json, CONFIG.clone()).path("$.[?(@.author ~= '.*(?i)d\\(Rees\\)')]")
        .expect("the path is correct");
}

fn regex_perf_test_before() {
    let json = Box::new(json!({
            "author":"abcd(Rees)",
        }));

    let _v = json.path("$.[?(@.author ~= '.*(?i)d\\(Rees\\)')]")
        .expect("the path is correct");
}

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("regex bench before", |b| b.iter(|| regex_perf_test_before()));
    c.bench_function("regex bench after", |b| {
        b.iter(|| regex_perf_test_after())
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);