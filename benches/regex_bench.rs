use std::str::FromStr;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde_json::{json, Value};
use jsonpath_rust::{cache_off, JsonPathFinder, JsonPathInst, JsonPathQuery};

fn regex_speed_test_after() {
    let json = Box::new(json!({
            "author":"abcd(Rees)",
        }));

    let _v = json.path("$.[?(@.author ~= '.*(?i)d\\(Rees\\)')]")
        .expect("the path is correct");
}

fn regex_speed_test_before() {
    cache_off();
    let json = Box::new(json!({
            "author":"abcd(Rees)",
        }));

    let _v = json.path("$.[?(@.author ~= '.*(?i)d\\(Rees\\)')]")
        .expect("the path is correct");
}

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("regex bench after", |b| b.iter(|| regex_speed_test_after()));
    c.bench_function("regex bench before", |b| b.iter(|| regex_speed_test_before()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);