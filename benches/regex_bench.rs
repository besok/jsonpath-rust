use std::str::FromStr;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde_json::{json, Value};
use jsonpath_rust::{JsonPathFinder, JsonPathInst, JsonPathQuery};

fn regex_speed_test() {
    let json = Box::new(json!({
            "author":"abcd(Rees)",
        }));

    let _v = json.path("$.[?(@.author ~= '.*(?i)d\\(Rees\\)')]")
        .expect("the path is correct");
}

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("regex bench", |b| b.iter(|| regex_speed_test()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);