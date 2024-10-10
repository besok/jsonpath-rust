use criterion::{criterion_group, criterion_main, Criterion};
use jsonpath_rust::{JsonPath, JsonPathQuery};
use serde_json::{json, Value};
use std::str::FromStr;

struct SearchData {
    json: serde_json::Value,
    path: JsonPath<Value>,
}

const PATH: &str = "$.[?(@.author ~= '.*(?i)d\\(Rees\\)')]";

fn regex_perf_test_with_reuse(cfg: &SearchData) {
    let _v = cfg.path.find(&cfg.json);
}

fn regex_perf_test_without_reuse() {
    let json = Box::new(json!({
        "author":"abcd(Rees)",
    }));

    let _v = json.path(PATH).expect("the path is correct");
}

fn json_path_compiling() {
    let _v = JsonPath::<Value>::from_str(PATH).unwrap();
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let data = SearchData {
        json: json!({
            "author":"abcd(Rees)",
        }),
        path: JsonPath::from_str(PATH).unwrap(),
    };
    c.bench_function("regex bench with reuse", |b| {
        b.iter(|| regex_perf_test_with_reuse(&data))
    });
    c.bench_function("regex bench without reuse", |b| {
        b.iter(regex_perf_test_without_reuse)
    });
    c.bench_function("JsonPath generation", |b| b.iter(json_path_compiling));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
