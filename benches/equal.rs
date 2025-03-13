use criterion::{criterion_group, criterion_main, Criterion};

use serde_json::{json, Value};
use std::str::FromStr;
use jsonpath_rust::JsonPath;
use jsonpath_rust::parser::model::JpQuery;
use jsonpath_rust::parser::parse_json_path;
use jsonpath_rust::query::Query;
use jsonpath_rust::query::state::State;

struct SearchData {
    json: Value,
    path: JpQuery,
}

const PATH: &str = "$[?@.author == 'abcd(Rees)']";

fn equal_perf_test_with_reuse(cfg: &SearchData) {
    let _v = cfg.path.process(State::root(&cfg.json)).data;
}
fn equal_perf_test_without_reuse() {
    let json = Box::new(json!({
        "author":"abcd(Rees)",
    }));

    let _v = json.query(PATH).expect("the path is correct");
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let data = SearchData {
        json: json!({
            "author":"abcd(Rees)",
        }),
        path: parse_json_path(PATH).unwrap(),
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
