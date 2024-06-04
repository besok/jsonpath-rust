use criterion::{criterion_group, criterion_main, Criterion};
use jsonpath_rust::{JsonPathInst, JsonPathQuery};
use serde_json::json;
use std::str::FromStr;

struct SearchData {
    json: serde_json::Value,
    path: JsonPathInst,
}

const PATH: &'static str = "$.[?(@.author ~= '.*(?i)d\\(Rees\\)')]";

fn regex_perf_test_with_reuse(cfg: &SearchData) {
    let _v = jsonpath_rust::find(&cfg.path, &cfg.json);
}

fn regex_perf_test_without_reuse() {
    let json = Box::new(json!({
        "author":"abcd(Rees)",
    }));

    let _v = json.path(PATH).expect("the path is correct");
}

fn json_path_inst_compiling() {
    let _v = JsonPathInst::from_str(PATH).unwrap();
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let data = SearchData {
        json: json!({
            "author":"abcd(Rees)",
        }),
        path: JsonPathInst::from_str(PATH).unwrap(),
    };
    c.bench_function("regex bench with reuse", |b| {
        b.iter(|| regex_perf_test_with_reuse(&data))
    });
    c.bench_function("regex bench without reuse", |b| {
        b.iter(|| regex_perf_test_without_reuse())
    });
    c.bench_function("JsonPathInst generation", |b| {
        b.iter(|| json_path_inst_compiling())
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
