use criterion::{criterion_group, criterion_main, Criterion};
use jsonpath_rust::parser::model::JpQuery;
use jsonpath_rust::parser::parse_json_path;
use jsonpath_rust::query::state::State;
use jsonpath_rust::query::{JsonPathEvalContext, Query};
use jsonpath_rust::JsonPath;
use serde_json::{json, Value};

struct SearchData {
    json: Value,
    path: JpQuery,
}

const PATH: &str = "$[?search(@.author,'.*(?i)d\\\\(Rees\\\\)')]";

fn regex_perf_test_with_reuse(cfg: &SearchData, ctx: &mut JsonPathEvalContext) {
    let _v = cfg
        .path
        .process_with_context(State::root(&cfg.json), ctx)
        .data;
}

fn regex_perf_test_without_reuse() {
    let json = Box::new(json!([{
        "author":"abcd(Rees)",
    }]));

    let _v = json.query(PATH).expect("the path is correct");
}

fn json_path_compiling() {
    let _v = parse_json_path(PATH).unwrap();
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let data = SearchData {
        json: json!([{
            "author":"abcd(Rees)",
        }]),
        path: parse_json_path(PATH).unwrap(),
    };
    let mut ctx = JsonPathEvalContext::with_regex_cache();
    c.bench_function("regex bench with reuse", |b| {
        b.iter(|| regex_perf_test_with_reuse(&data, &mut ctx))
    });
    c.bench_function("regex bench without reuse", |b| {
        b.iter(regex_perf_test_without_reuse)
    });
    c.bench_function("JsonPath generation", |b| b.iter(json_path_compiling));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
