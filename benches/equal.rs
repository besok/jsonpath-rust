use criterion::{criterion_group, criterion_main, Criterion};
use jsonpath_rust::{JsonPathInst, JsonPathQuery};
use serde_json::json;
use std::str::FromStr;

struct SearchData {
    json: serde_json::Value,
    path: JsonPathInst,
}

const PATH: &str = "$.[?(@.author == 'abcd(Rees)')]";

fn equal_perf_test_with_reuse(cfg: &SearchData) {
    let _v = jsonpath_rust::find(&cfg.path, &cfg.json);
}

fn equal_perf_test_without_reuse() {
    let json = Box::new(json!({
        "author":"abcd(Rees)",
    }));

    let _v = json.path(PATH).expect("the path is correct");
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let data = SearchData {
        json: json!({
            "author":"abcd(Rees)",
        }),
        path: JsonPathInst::from_str(PATH).unwrap(),
    };
    c.bench_function("equal bench with reuse", |b| {
        b.iter(|| equal_perf_test_with_reuse(&data))
    });
    c.bench_function("equal bench without reuse", |b| {
        b.iter(equal_perf_test_without_reuse)
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
